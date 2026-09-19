//! Mail from a linked mailbox: templates, a send that is recorded whatever
//! the provider said, the allowlist that stops a test reaching anyone real,
//! and a reply that comes back with its attachment.

use tbd_proto::finance::v1::{
    DeleteMailTemplateRequest, GetMailRequest, ListMailRequest, ListMailTemplatesRequest,
    SendMailRequest, UploadDocumentRequest, UpsertMailTemplateRequest,
};
use tonic::Code;

use crate::{
    connectors::{OWNER, READER, as_caller, kinds_mail, link_with, seed, sync_and_wait},
    support::start_with_kinds,
};

#[tokio::test]
async fn a_template_belongs_to_a_party_and_is_named_once() {
    let (factory, _, _) = kinds_mail(false, false);
    let (server, pool) = start_with_kinds(factory).await;
    let (personal, company) = seed(&pool).await;
    let mut c = server.client().await;
    let input = |party| UpsertMailTemplateRequest {
        party_id: party,
        name: "Monthly bundle".into(),
        subject: "Računi {{MonthName}} {{Year}}".into(),
        body: "Bok,\n\nu privitku računi za {{Month}}/{{Year}}.\n\n{{Company}}".into(),
        to: vec![
            "books@accountant.test".into(),
            " books@accountant.test".into(),
        ],
        cc: vec!["nevio@inorbit.hr".into()],
        bcc: vec![],
        ..UpsertMailTemplateRequest::default()
    };

    // The reader may see the company, not the person.
    let e = c
        .upsert_mail_template(as_caller(READER, input(personal.to_string())))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");
    let made = c
        .upsert_mail_template(as_caller(OWNER, input(company.to_string())))
        .await
        .unwrap()
        .into_inner()
        .template
        .unwrap();
    assert_eq!(
        made.to,
        vec!["books@accountant.test"],
        "deduplicated, trimmed"
    );
    let e = c
        .upsert_mail_template(as_caller(OWNER, input(company.to_string())))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::InvalidArgument, "same name twice: {e}");
    let listed = c
        .list_mail_templates(as_caller(READER, ListMailTemplatesRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .templates;
    assert_eq!(listed.len(), 1);
    c.delete_mail_template(as_caller(
        OWNER,
        DeleteMailTemplateRequest {
            id: made.id.clone(),
        },
    ))
    .await
    .unwrap();
    let listed = c
        .list_mail_templates(as_caller(OWNER, ListMailTemplatesRequest::default()))
        .await
        .unwrap()
        .into_inner()
        .templates;
    assert!(listed.is_empty());
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn a_send_is_guarded_recorded_and_answered() {
    let (factory, _, sent) = kinds_mail(false, false);
    let (server, pool) = start_with_kinds(factory).await;
    let (_, company) = seed(&pool).await;
    let mailbox = link_with(&server, company, "ok").await;
    assert!(mailbox.can_send, "the consent included sending");
    let mut c = server.client().await;

    // Something to attach: an uploaded receipt.
    let doc = c
        .upload_document(as_caller(
            OWNER,
            UploadDocumentRequest {
                party_id: company.to_string(),
                filename: "openai-august.pdf".into(),
                content_type: "application/pdf".into(),
                bytes: b"%PDF-1.4 openai".to_vec(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .document
        .unwrap();
    let mail = |to: &str, subject: &str| SendMailRequest {
        connector_id: mailbox.id.clone(),
        to: vec![to.into()],
        cc: vec![],
        bcc: vec!["nevio.vesic@gmail.com".into()],
        subject: subject.into(),
        body: "u privitku".into(),
        attachment_document_ids: vec![doc.id.clone()],
        ..SendMailRequest::default()
    };

    // The local environment allows only the owner's own two addresses: a
    // real accountant is never reached from a test, and the provider is
    // never asked.
    let e = c
        .send_mail(as_caller(
            OWNER,
            mail("books@accountant.test", "Računi 8/2026"),
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition, "{e}");
    assert!(
        e.message().contains("not an allowed recipient"),
        "{}",
        e.message()
    );
    assert!(sent.lock().unwrap().is_empty(), "nothing went out");
    let e = c
        .send_mail(as_caller(OWNER, mail("not an address", "x")))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition, "{e}");
    // The reader may see the company but not send as its mailbox? It may:
    // the grant is the grant. A stranger's mailbox is not found.
    let e = c
        .send_mail(as_caller(
            OWNER,
            SendMailRequest {
                connector_id: uuid::Uuid::new_v4().to_string(),
                ..mail("nevio@inorbit.hr", "x")
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::NotFound, "{e}");

    // Sent: recorded with recipients, attachment and the provider's ids.
    let out = c
        .send_mail(as_caller(OWNER, mail("nevio@inorbit.hr", "Računi 8/2026")))
        .await
        .unwrap()
        .into_inner()
        .mail
        .unwrap();
    assert_eq!(out.status, "sent");
    assert_eq!(out.direction, "out");
    assert_eq!(out.from, "inbox@example.test");
    assert_eq!(out.to, vec!["nevio@inorbit.hr"]);
    assert_eq!(out.bcc, vec!["nevio.vesic@gmail.com"]);
    assert_eq!(out.documents.len(), 1);
    assert_eq!(out.documents[0].filename, "openai-august.pdf");
    assert!(!out.thread_key.is_empty());
    {
        let went = sent.lock().unwrap();
        assert_eq!(went.len(), 1);
        assert_eq!(went[0].attachments[0].bytes, b"%PDF-1.4 openai");
        assert_eq!(went[0].subject, "Računi 8/2026");
    }

    // A provider failure is a failed row, not an error.
    let failed = c
        .send_mail(as_caller(OWNER, mail("nevio@inorbit.hr", "FAIL please")))
        .await
        .unwrap()
        .into_inner()
        .mail
        .unwrap();
    assert_eq!(failed.status, "failed");
    assert!(failed.error.contains("refused"), "{}", failed.error);

    // The next pull brings the reply, with its PDF as a document of the party.
    let run = sync_and_wait(&mut c, &mailbox.id).await;
    assert_eq!(run.outcome, "ok", "{}", run.error);
    let got = c
        .get_mail(as_caller(OWNER, GetMailRequest { id: out.id.clone() }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(got.thread.len(), 2, "the mail and its reply");
    let reply = &got.thread[1];
    assert_eq!(reply.direction, "in");
    assert_eq!(reply.status, "received");
    assert_eq!(reply.parent_id, out.id);
    assert!(reply.from.contains("books@accountant.test"));
    assert_eq!(reply.documents.len(), 1);
    assert_eq!(reply.documents[0].filename, "confirmation.pdf");
    // Asked again, the same reply is not imported twice.
    let run = sync_and_wait(&mut c, &mailbox.id).await;
    assert_eq!(run.outcome, "ok", "{}", run.error);
    let listed = c
        .list_mail(as_caller(OWNER, ListMailRequest::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(listed.total, 3, "sent, failed, and one reply");
    let outgoing = listed.mails.iter().find(|m| m.id == out.id).unwrap();
    assert_eq!(outgoing.replies, 1);
    // The reader sees the company's mail; the reply's PDF is a receipt of the company.
    let seen = c
        .list_mail(as_caller(
            READER,
            ListMailRequest {
                q: "confirmation".into(),
                ..ListMailRequest::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(seen.total, 1);

    // A mailbox linked without send consent cannot send, and says why.
    let quiet = link_with(&server, company, "nosend").await;
    assert!(!quiet.can_send);
    let e = c
        .send_mail(as_caller(
            OWNER,
            SendMailRequest {
                connector_id: quiet.id.clone(),
                ..mail("nevio@inorbit.hr", "x")
            },
        ))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::FailedPrecondition, "{e}");
    assert!(e.message().contains("allow sending"), "{}", e.message());
}
