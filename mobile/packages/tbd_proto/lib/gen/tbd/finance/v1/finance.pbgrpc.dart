// This is a generated file - do not edit.
//
// Generated from tbd/finance/v1/finance.proto.

// @dart = 3.3

// ignore_for_file: annotate_overrides, camel_case_types, comment_references
// ignore_for_file: constant_identifier_names
// ignore_for_file: curly_braces_in_flow_control_structures
// ignore_for_file: deprecated_member_use_from_same_package, library_prefixes
// ignore_for_file: non_constant_identifier_names, prefer_relative_imports

import 'dart:async' as $async;
import 'dart:core' as $core;

import 'package:grpc/service_api.dart' as $grpc;
import 'package:protobuf/protobuf.dart' as $pb;

import 'finance.pb.dart' as $0;

export 'finance.pb.dart';

/// The finance service. Ping is the liveness RPC every service carries; real
/// RPCs are added beside it.
@$pb.GrpcServiceName('tbd.finance.v1.FinanceService')
class FinanceServiceClient extends $grpc.Client {
  /// The hostname for this service.
  static const $core.String defaultHost = '';

  /// OAuth scopes needed for the client.
  static const $core.List<$core.String> oauthScopes = [
    '',
  ];

  FinanceServiceClient(super.channel, {super.options, super.interceptors});

  /// Echo a message. Proves configuration, telemetry and fault injection reach
  /// an RPC.
  $grpc.ResponseFuture<$0.PingResponse> ping(
    $0.PingRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$ping, request, options: options);
  }

  /// Transactions the caller is allowed to see.
  ///
  /// There is deliberately no party filter in the request. The set of parties a
  /// caller may read is derived from the token Envoy verified, never from
  /// anything the caller sends: a request naming someone else's party could not
  /// be told apart from an honest one. `party_ids` narrows *within* that set and
  /// can only ever shrink it.
  $grpc.ResponseFuture<$0.ListTransactionsResponse> listTransactions(
    $0.ListTransactionsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listTransactions, request, options: options);
  }

  /// One transaction with everything the bank sent about it, including the
  /// raw record. Outside the caller's grant it is NOT_FOUND, never
  /// PERMISSION_DENIED.
  $grpc.ResponseFuture<$0.GetTransactionResponse> getTransaction(
    $0.GetTransactionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getTransaction, request, options: options);
  }

  /// Money in and out by month and category, for the parties the caller may
  /// read. Grouped by currency as well: a cross-currency total is never
  /// computed, because it would need a rate, and a rate needs a date and a
  /// source. Transfers between the caller's own accounts are excluded unless
  /// asked for -- they are real on both sides, but counting them in a combined
  /// view is the same euros twice.
  $grpc.ResponseFuture<$0.MonthlySummaryResponse> monthlySummary(
    $0.MonthlySummaryRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$monthlySummary, request, options: options);
  }

  /// The parties the caller may read: what the personal / business / combined
  /// toggle chooses between. Never more than the grant.
  $grpc.ResponseFuture<$0.ListPartiesResponse> listParties(
    $0.ListPartiesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listParties, request, options: options);
  }

  /// Bank accounts and their sync state.
  $grpc.ResponseFuture<$0.ListAccountsResponse> listAccounts(
    $0.ListAccountsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listAccounts, request, options: options);
  }

  /// Fetch one account from the bank now, from the reserve the scheduler
  /// never spends. Refused without a request when the day's allowance is
  /// gone or the bank has asked us to wait.
  $grpc.ResponseFuture<$0.RefreshAccountResponse> refreshAccount(
    $0.RefreshAccountRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$refreshAccount, request, options: options);
  }

  /// Whether the scheduler fetches this account. Off for an account that
  /// never moves: every scheduled fetch is one of the day's four.
  $grpc.ResponseFuture<$0.SetAccountSyncResponse> setAccountSync(
    $0.SetAccountSyncRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$setAccountSync, request, options: options);
  }

  /// Categories for the caller's parties.
  $grpc.ResponseFuture<$0.ListCategoriesResponse> listCategories(
    $0.ListCategoriesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listCategories, request, options: options);
  }

  /// Set a transaction's category by hand. A person's decision beats every
  /// rule, now and on every later pass.
  $grpc.ResponseFuture<$0.DeclareCategoryResponse> declareCategory(
    $0.DeclareCategoryRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$declareCategory, request, options: options);
  }

  /// Create or change a category of a party the caller may read. Archiving
  /// one disables its rules and reapplies the rest; rows already declared
  /// under it keep it.
  $grpc.ResponseFuture<$0.UpsertCategoryResponse> upsertCategory(
    $0.UpsertCategoryRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$upsertCategory, request, options: options);
  }

  /// The rules that categorise transactions, in the order they apply.
  $grpc.ResponseFuture<$0.ListRulesResponse> listRules(
    $0.ListRulesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listRules, request, options: options);
  }

  /// Create or change a rule, then reapply every rule for the party. A pass is
  /// a pure function of the rules, so the answer after this call is the same
  /// one the next sync would reach.
  $grpc.ResponseFuture<$0.UpsertRuleResponse> upsertRule(
    $0.UpsertRuleRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$upsertRule, request, options: options);
  }

  /// Begin linking a bank: returns the URL the person must visit.
  $grpc.ResponseFuture<$0.StartConnectionResponse> startConnection(
    $0.StartConnectionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$startConnection, request, options: options);
  }

  /// Finish linking with what the bank's redirect carried. The code travels in
  /// the body, never a query string; a state that is not the caller's is
  /// NOT_FOUND, never PERMISSION_DENIED, which would confirm it exists.
  $grpc.ResponseFuture<$0.CompleteConnectionResponse> completeConnection(
    $0.CompleteConnectionRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$completeConnection, request, options: options);
  }

  /// Every bank link the caller may see, and its state.
  $grpc.ResponseFuture<$0.ListConnectionsResponse> listConnections(
    $0.ListConnectionsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listConnections, request, options: options);
  }

  /// The issuing profile of a party: what the footer of a Croatian invoice
  /// must carry. Absent until set; a draft cannot be created without it.
  $grpc.ResponseFuture<$0.GetIssuerResponse> getIssuer(
    $0.GetIssuerRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getIssuer, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpsertIssuerResponse> upsertIssuer(
    $0.UpsertIssuerRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$upsertIssuer, request, options: options);
  }

  /// Who we invoice.
  $grpc.ResponseFuture<$0.ListClientsResponse> listClients(
    $0.ListClientsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listClients, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpsertClientResponse> upsertClient(
    $0.UpsertClientRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$upsertClient, request, options: options);
  }

  /// Every invoice the caller may see, newest first.
  $grpc.ResponseFuture<$0.ListInvoicesResponse> listInvoices(
    $0.ListInvoicesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listInvoices, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetInvoiceResponse> getInvoice(
    $0.GetInvoiceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getInvoice, request, options: options);
  }

  /// A new draft for a client, pre-filled from the last approved invoice to
  /// them. No number is taken.
  $grpc.ResponseFuture<$0.CreateInvoiceResponse> createInvoice(
    $0.CreateInvoiceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$createInvoice, request, options: options);
  }

  /// Replace a draft's dates, note and lines. Totals are recomputed here,
  /// never trusted from the client.
  $grpc.ResponseFuture<$0.UpdateInvoiceResponse> updateInvoice(
    $0.UpdateInvoiceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateInvoice, request, options: options);
  }

  /// Render the draft as it would be approved now: next number, current
  /// minute, watermarked. Writes nothing. Returns the hash `ApproveInvoice`
  /// must be given back, and the PDF.
  $grpc.ResponseFuture<$0.PreviewInvoiceResponse> previewInvoice(
    $0.PreviewInvoiceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$previewInvoice, request, options: options);
  }

  /// Approve: allocate the number, render, store. The content hash must be
  /// the one the preview returned; a draft changed since is
  /// FAILED_PRECONDITION and nothing is written.
  $grpc.ResponseFuture<$0.ApproveInvoiceResponse> approveInvoice(
    $0.ApproveInvoiceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$approveInvoice, request, options: options);
  }

  /// Cancel a draft or an approved invoice. An approved one keeps its number.
  $grpc.ResponseFuture<$0.CancelInvoiceResponse> cancelInvoice(
    $0.CancelInvoiceRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$cancelInvoice, request, options: options);
  }

  /// The stored PDF of an approved invoice, base64.
  $grpc.ResponseFuture<$0.GetInvoiceDocumentResponse> getInvoiceDocument(
    $0.GetInvoiceDocumentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getInvoiceDocument, request, options: options);
  }

  /// What a client is usually billed for: the rows a draft starts from.
  $grpc.ResponseFuture<$0.ListLineTemplatesResponse> listLineTemplates(
    $0.ListLineTemplatesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listLineTemplates, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpsertLineTemplateResponse> upsertLineTemplate(
    $0.UpsertLineTemplateRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$upsertLineTemplate, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteLineTemplateResponse> deleteLineTemplate(
    $0.DeleteLineTemplateRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteLineTemplate, request, options: options);
  }

  /// Every kind the service knows, whether or not it is configured.
  $grpc.ResponseFuture<$0.ListConnectorKindsResponse> listConnectorKinds(
    $0.ListConnectorKindsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listConnectorKinds, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListConnectorsResponse> listConnectors(
    $0.ListConnectorsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listConnectors, request, options: options);
  }

  /// Every connector in view with its latest run, then each one again
  /// whenever it changes: a pull's progress, its outcome, a relink, a
  /// removal. The page draws from this instead of polling.
  $grpc.ResponseStream<$0.WatchConnectorsResponse> watchConnectors(
    $0.WatchConnectorsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createStreamingCall(
        _$watchConnectors, $async.Stream.fromIterable([request]),
        options: options);
  }

  /// Begin a link: the URL to send the person to.
  $grpc.ResponseFuture<$0.StartConnectorResponse> startConnector(
    $0.StartConnectorRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$startConnector, request, options: options);
  }

  /// Finish with what the provider's redirect carried. The code is in the
  /// body; a state that is not the caller's is NOT_FOUND.
  $grpc.ResponseFuture<$0.CompleteConnectorResponse> completeConnector(
    $0.CompleteConnectorRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$completeConnector, request, options: options);
  }

  /// Prove the link still works.
  $grpc.ResponseFuture<$0.TestConnectorResponse> testConnector(
    $0.TestConnectorRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$testConnector, request, options: options);
  }

  /// Pull new documents now.
  $grpc.ResponseFuture<$0.SyncConnectorResponse> syncConnector(
    $0.SyncConnectorRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$syncConnector, request, options: options);
  }

  /// Change the non-secret settings (the query).
  $grpc.ResponseFuture<$0.ConfigureConnectorResponse> configureConnector(
    $0.ConfigureConnectorRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$configureConnector, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteConnectorResponse> deleteConnector(
    $0.DeleteConnectorRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteConnector, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListConnectorRunsResponse> listConnectorRuns(
    $0.ListConnectorRunsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listConnectorRuns, request, options: options);
  }

  /// Mail sent from a linked mailbox whose consent included sending, and
  /// the replies that came back. Templates hold the recurring shape with
  /// helpers the page renders; the service stores what was sent.
  $grpc.ResponseFuture<$0.ListMailTemplatesResponse> listMailTemplates(
    $0.ListMailTemplatesRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listMailTemplates, request, options: options);
  }

  $grpc.ResponseFuture<$0.UpsertMailTemplateResponse> upsertMailTemplate(
    $0.UpsertMailTemplateRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$upsertMailTemplate, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteMailTemplateResponse> deleteMailTemplate(
    $0.DeleteMailTemplateRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteMailTemplate, request, options: options);
  }

  /// Send as a connector and record the mail, sent or failed. Refused before
  /// the provider is asked when the connector cannot send, an address is
  /// malformed, or the environment's allowlist excludes one.
  $grpc.ResponseFuture<$0.SendMailResponse> sendMail(
    $0.SendMailRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$sendMail, request, options: options);
  }

  $grpc.ResponseFuture<$0.ListMailResponse> listMail(
    $0.ListMailRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listMail, request, options: options);
  }

  /// One mail with its thread: what it answers and every reply, oldest first.
  $grpc.ResponseFuture<$0.GetMailResponse> getMail(
    $0.GetMailRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getMail, request, options: options);
  }

  /// Receipts and other pulled documents, newest first, searchable.
  $grpc.ResponseFuture<$0.ListDocumentsResponse> listDocuments(
    $0.ListDocumentsRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$listDocuments, request, options: options);
  }

  /// A month of one party's transactions, each with what the accountant
  /// needs for it and which receipt covers it. Runs the matcher first, so
  /// a receipt pulled since is linked before the page sees it.
  $grpc.ResponseFuture<$0.MonthlyReconciliationResponse> monthlyReconciliation(
    $0.MonthlyReconciliationRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$monthlyReconciliation, request, options: options);
  }

  /// A person links a receipt to the transaction that paid it.
  $grpc.ResponseFuture<$0.LinkDocumentResponse> linkDocument(
    $0.LinkDocumentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$linkDocument, request, options: options);
  }

  $grpc.ResponseFuture<$0.UnlinkDocumentResponse> unlinkDocument(
    $0.UnlinkDocumentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$unlinkDocument, request, options: options);
  }

  /// A person's note on a transaction for the accountant; empty removes it.
  $grpc.ResponseFuture<$0.SetTransactionNoteResponse> setTransactionNote(
    $0.SetTransactionNoteRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$setTransactionNote, request, options: options);
  }

  /// A person's word on a counterparty: what the accountant needs for it.
  $grpc.ResponseFuture<$0.SetCounterpartyPolicyResponse> setCounterpartyPolicy(
    $0.SetCounterpartyPolicyRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$setCounterpartyPolicy, request, options: options);
  }

  $grpc.ResponseFuture<$0.DeleteCounterpartyPolicyResponse>
      deleteCounterpartyPolicy(
    $0.DeleteCounterpartyPolicyRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$deleteCounterpartyPolicy, request,
        options: options);
  }

  /// Set what a receipt says, by hand. Declared fields survive a re-read.
  $grpc.ResponseFuture<$0.UpdateDocumentResponse> updateDocument(
    $0.UpdateDocumentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$updateDocument, request, options: options);
  }

  /// Read the document's text again and refill what was not declared.
  $grpc.ResponseFuture<$0.ExtractDocumentResponse> extractDocument(
    $0.ExtractDocumentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$extractDocument, request, options: options);
  }

  $grpc.ResponseFuture<$0.GetDocumentResponse> getDocument(
    $0.GetDocumentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$getDocument, request, options: options);
  }

  /// A receipt a person adds by hand: a PDF from a vendor's portal, or a
  /// photo of a paper receipt. Whose it is is the caller's word and final.
  /// The same bytes uploaded twice are one document.
  $grpc.ResponseFuture<$0.UploadDocumentResponse> uploadDocument(
    $0.UploadDocumentRequest request, {
    $grpc.CallOptions? options,
  }) {
    return $createUnaryCall(_$uploadDocument, request, options: options);
  }

  // method descriptors

  static final _$ping = $grpc.ClientMethod<$0.PingRequest, $0.PingResponse>(
      '/tbd.finance.v1.FinanceService/Ping',
      ($0.PingRequest value) => value.writeToBuffer(),
      $0.PingResponse.fromBuffer);
  static final _$listTransactions = $grpc.ClientMethod<
          $0.ListTransactionsRequest, $0.ListTransactionsResponse>(
      '/tbd.finance.v1.FinanceService/ListTransactions',
      ($0.ListTransactionsRequest value) => value.writeToBuffer(),
      $0.ListTransactionsResponse.fromBuffer);
  static final _$getTransaction =
      $grpc.ClientMethod<$0.GetTransactionRequest, $0.GetTransactionResponse>(
          '/tbd.finance.v1.FinanceService/GetTransaction',
          ($0.GetTransactionRequest value) => value.writeToBuffer(),
          $0.GetTransactionResponse.fromBuffer);
  static final _$monthlySummary =
      $grpc.ClientMethod<$0.MonthlySummaryRequest, $0.MonthlySummaryResponse>(
          '/tbd.finance.v1.FinanceService/MonthlySummary',
          ($0.MonthlySummaryRequest value) => value.writeToBuffer(),
          $0.MonthlySummaryResponse.fromBuffer);
  static final _$listParties =
      $grpc.ClientMethod<$0.ListPartiesRequest, $0.ListPartiesResponse>(
          '/tbd.finance.v1.FinanceService/ListParties',
          ($0.ListPartiesRequest value) => value.writeToBuffer(),
          $0.ListPartiesResponse.fromBuffer);
  static final _$listAccounts =
      $grpc.ClientMethod<$0.ListAccountsRequest, $0.ListAccountsResponse>(
          '/tbd.finance.v1.FinanceService/ListAccounts',
          ($0.ListAccountsRequest value) => value.writeToBuffer(),
          $0.ListAccountsResponse.fromBuffer);
  static final _$refreshAccount =
      $grpc.ClientMethod<$0.RefreshAccountRequest, $0.RefreshAccountResponse>(
          '/tbd.finance.v1.FinanceService/RefreshAccount',
          ($0.RefreshAccountRequest value) => value.writeToBuffer(),
          $0.RefreshAccountResponse.fromBuffer);
  static final _$setAccountSync =
      $grpc.ClientMethod<$0.SetAccountSyncRequest, $0.SetAccountSyncResponse>(
          '/tbd.finance.v1.FinanceService/SetAccountSync',
          ($0.SetAccountSyncRequest value) => value.writeToBuffer(),
          $0.SetAccountSyncResponse.fromBuffer);
  static final _$listCategories =
      $grpc.ClientMethod<$0.ListCategoriesRequest, $0.ListCategoriesResponse>(
          '/tbd.finance.v1.FinanceService/ListCategories',
          ($0.ListCategoriesRequest value) => value.writeToBuffer(),
          $0.ListCategoriesResponse.fromBuffer);
  static final _$declareCategory =
      $grpc.ClientMethod<$0.DeclareCategoryRequest, $0.DeclareCategoryResponse>(
          '/tbd.finance.v1.FinanceService/DeclareCategory',
          ($0.DeclareCategoryRequest value) => value.writeToBuffer(),
          $0.DeclareCategoryResponse.fromBuffer);
  static final _$upsertCategory =
      $grpc.ClientMethod<$0.UpsertCategoryRequest, $0.UpsertCategoryResponse>(
          '/tbd.finance.v1.FinanceService/UpsertCategory',
          ($0.UpsertCategoryRequest value) => value.writeToBuffer(),
          $0.UpsertCategoryResponse.fromBuffer);
  static final _$listRules =
      $grpc.ClientMethod<$0.ListRulesRequest, $0.ListRulesResponse>(
          '/tbd.finance.v1.FinanceService/ListRules',
          ($0.ListRulesRequest value) => value.writeToBuffer(),
          $0.ListRulesResponse.fromBuffer);
  static final _$upsertRule =
      $grpc.ClientMethod<$0.UpsertRuleRequest, $0.UpsertRuleResponse>(
          '/tbd.finance.v1.FinanceService/UpsertRule',
          ($0.UpsertRuleRequest value) => value.writeToBuffer(),
          $0.UpsertRuleResponse.fromBuffer);
  static final _$startConnection =
      $grpc.ClientMethod<$0.StartConnectionRequest, $0.StartConnectionResponse>(
          '/tbd.finance.v1.FinanceService/StartConnection',
          ($0.StartConnectionRequest value) => value.writeToBuffer(),
          $0.StartConnectionResponse.fromBuffer);
  static final _$completeConnection = $grpc.ClientMethod<
          $0.CompleteConnectionRequest, $0.CompleteConnectionResponse>(
      '/tbd.finance.v1.FinanceService/CompleteConnection',
      ($0.CompleteConnectionRequest value) => value.writeToBuffer(),
      $0.CompleteConnectionResponse.fromBuffer);
  static final _$listConnections =
      $grpc.ClientMethod<$0.ListConnectionsRequest, $0.ListConnectionsResponse>(
          '/tbd.finance.v1.FinanceService/ListConnections',
          ($0.ListConnectionsRequest value) => value.writeToBuffer(),
          $0.ListConnectionsResponse.fromBuffer);
  static final _$getIssuer =
      $grpc.ClientMethod<$0.GetIssuerRequest, $0.GetIssuerResponse>(
          '/tbd.finance.v1.FinanceService/GetIssuer',
          ($0.GetIssuerRequest value) => value.writeToBuffer(),
          $0.GetIssuerResponse.fromBuffer);
  static final _$upsertIssuer =
      $grpc.ClientMethod<$0.UpsertIssuerRequest, $0.UpsertIssuerResponse>(
          '/tbd.finance.v1.FinanceService/UpsertIssuer',
          ($0.UpsertIssuerRequest value) => value.writeToBuffer(),
          $0.UpsertIssuerResponse.fromBuffer);
  static final _$listClients =
      $grpc.ClientMethod<$0.ListClientsRequest, $0.ListClientsResponse>(
          '/tbd.finance.v1.FinanceService/ListClients',
          ($0.ListClientsRequest value) => value.writeToBuffer(),
          $0.ListClientsResponse.fromBuffer);
  static final _$upsertClient =
      $grpc.ClientMethod<$0.UpsertClientRequest, $0.UpsertClientResponse>(
          '/tbd.finance.v1.FinanceService/UpsertClient',
          ($0.UpsertClientRequest value) => value.writeToBuffer(),
          $0.UpsertClientResponse.fromBuffer);
  static final _$listInvoices =
      $grpc.ClientMethod<$0.ListInvoicesRequest, $0.ListInvoicesResponse>(
          '/tbd.finance.v1.FinanceService/ListInvoices',
          ($0.ListInvoicesRequest value) => value.writeToBuffer(),
          $0.ListInvoicesResponse.fromBuffer);
  static final _$getInvoice =
      $grpc.ClientMethod<$0.GetInvoiceRequest, $0.GetInvoiceResponse>(
          '/tbd.finance.v1.FinanceService/GetInvoice',
          ($0.GetInvoiceRequest value) => value.writeToBuffer(),
          $0.GetInvoiceResponse.fromBuffer);
  static final _$createInvoice =
      $grpc.ClientMethod<$0.CreateInvoiceRequest, $0.CreateInvoiceResponse>(
          '/tbd.finance.v1.FinanceService/CreateInvoice',
          ($0.CreateInvoiceRequest value) => value.writeToBuffer(),
          $0.CreateInvoiceResponse.fromBuffer);
  static final _$updateInvoice =
      $grpc.ClientMethod<$0.UpdateInvoiceRequest, $0.UpdateInvoiceResponse>(
          '/tbd.finance.v1.FinanceService/UpdateInvoice',
          ($0.UpdateInvoiceRequest value) => value.writeToBuffer(),
          $0.UpdateInvoiceResponse.fromBuffer);
  static final _$previewInvoice =
      $grpc.ClientMethod<$0.PreviewInvoiceRequest, $0.PreviewInvoiceResponse>(
          '/tbd.finance.v1.FinanceService/PreviewInvoice',
          ($0.PreviewInvoiceRequest value) => value.writeToBuffer(),
          $0.PreviewInvoiceResponse.fromBuffer);
  static final _$approveInvoice =
      $grpc.ClientMethod<$0.ApproveInvoiceRequest, $0.ApproveInvoiceResponse>(
          '/tbd.finance.v1.FinanceService/ApproveInvoice',
          ($0.ApproveInvoiceRequest value) => value.writeToBuffer(),
          $0.ApproveInvoiceResponse.fromBuffer);
  static final _$cancelInvoice =
      $grpc.ClientMethod<$0.CancelInvoiceRequest, $0.CancelInvoiceResponse>(
          '/tbd.finance.v1.FinanceService/CancelInvoice',
          ($0.CancelInvoiceRequest value) => value.writeToBuffer(),
          $0.CancelInvoiceResponse.fromBuffer);
  static final _$getInvoiceDocument = $grpc.ClientMethod<
          $0.GetInvoiceDocumentRequest, $0.GetInvoiceDocumentResponse>(
      '/tbd.finance.v1.FinanceService/GetInvoiceDocument',
      ($0.GetInvoiceDocumentRequest value) => value.writeToBuffer(),
      $0.GetInvoiceDocumentResponse.fromBuffer);
  static final _$listLineTemplates = $grpc.ClientMethod<
          $0.ListLineTemplatesRequest, $0.ListLineTemplatesResponse>(
      '/tbd.finance.v1.FinanceService/ListLineTemplates',
      ($0.ListLineTemplatesRequest value) => value.writeToBuffer(),
      $0.ListLineTemplatesResponse.fromBuffer);
  static final _$upsertLineTemplate = $grpc.ClientMethod<
          $0.UpsertLineTemplateRequest, $0.UpsertLineTemplateResponse>(
      '/tbd.finance.v1.FinanceService/UpsertLineTemplate',
      ($0.UpsertLineTemplateRequest value) => value.writeToBuffer(),
      $0.UpsertLineTemplateResponse.fromBuffer);
  static final _$deleteLineTemplate = $grpc.ClientMethod<
          $0.DeleteLineTemplateRequest, $0.DeleteLineTemplateResponse>(
      '/tbd.finance.v1.FinanceService/DeleteLineTemplate',
      ($0.DeleteLineTemplateRequest value) => value.writeToBuffer(),
      $0.DeleteLineTemplateResponse.fromBuffer);
  static final _$listConnectorKinds = $grpc.ClientMethod<
          $0.ListConnectorKindsRequest, $0.ListConnectorKindsResponse>(
      '/tbd.finance.v1.FinanceService/ListConnectorKinds',
      ($0.ListConnectorKindsRequest value) => value.writeToBuffer(),
      $0.ListConnectorKindsResponse.fromBuffer);
  static final _$listConnectors =
      $grpc.ClientMethod<$0.ListConnectorsRequest, $0.ListConnectorsResponse>(
          '/tbd.finance.v1.FinanceService/ListConnectors',
          ($0.ListConnectorsRequest value) => value.writeToBuffer(),
          $0.ListConnectorsResponse.fromBuffer);
  static final _$watchConnectors =
      $grpc.ClientMethod<$0.WatchConnectorsRequest, $0.WatchConnectorsResponse>(
          '/tbd.finance.v1.FinanceService/WatchConnectors',
          ($0.WatchConnectorsRequest value) => value.writeToBuffer(),
          $0.WatchConnectorsResponse.fromBuffer);
  static final _$startConnector =
      $grpc.ClientMethod<$0.StartConnectorRequest, $0.StartConnectorResponse>(
          '/tbd.finance.v1.FinanceService/StartConnector',
          ($0.StartConnectorRequest value) => value.writeToBuffer(),
          $0.StartConnectorResponse.fromBuffer);
  static final _$completeConnector = $grpc.ClientMethod<
          $0.CompleteConnectorRequest, $0.CompleteConnectorResponse>(
      '/tbd.finance.v1.FinanceService/CompleteConnector',
      ($0.CompleteConnectorRequest value) => value.writeToBuffer(),
      $0.CompleteConnectorResponse.fromBuffer);
  static final _$testConnector =
      $grpc.ClientMethod<$0.TestConnectorRequest, $0.TestConnectorResponse>(
          '/tbd.finance.v1.FinanceService/TestConnector',
          ($0.TestConnectorRequest value) => value.writeToBuffer(),
          $0.TestConnectorResponse.fromBuffer);
  static final _$syncConnector =
      $grpc.ClientMethod<$0.SyncConnectorRequest, $0.SyncConnectorResponse>(
          '/tbd.finance.v1.FinanceService/SyncConnector',
          ($0.SyncConnectorRequest value) => value.writeToBuffer(),
          $0.SyncConnectorResponse.fromBuffer);
  static final _$configureConnector = $grpc.ClientMethod<
          $0.ConfigureConnectorRequest, $0.ConfigureConnectorResponse>(
      '/tbd.finance.v1.FinanceService/ConfigureConnector',
      ($0.ConfigureConnectorRequest value) => value.writeToBuffer(),
      $0.ConfigureConnectorResponse.fromBuffer);
  static final _$deleteConnector =
      $grpc.ClientMethod<$0.DeleteConnectorRequest, $0.DeleteConnectorResponse>(
          '/tbd.finance.v1.FinanceService/DeleteConnector',
          ($0.DeleteConnectorRequest value) => value.writeToBuffer(),
          $0.DeleteConnectorResponse.fromBuffer);
  static final _$listConnectorRuns = $grpc.ClientMethod<
          $0.ListConnectorRunsRequest, $0.ListConnectorRunsResponse>(
      '/tbd.finance.v1.FinanceService/ListConnectorRuns',
      ($0.ListConnectorRunsRequest value) => value.writeToBuffer(),
      $0.ListConnectorRunsResponse.fromBuffer);
  static final _$listMailTemplates = $grpc.ClientMethod<
          $0.ListMailTemplatesRequest, $0.ListMailTemplatesResponse>(
      '/tbd.finance.v1.FinanceService/ListMailTemplates',
      ($0.ListMailTemplatesRequest value) => value.writeToBuffer(),
      $0.ListMailTemplatesResponse.fromBuffer);
  static final _$upsertMailTemplate = $grpc.ClientMethod<
          $0.UpsertMailTemplateRequest, $0.UpsertMailTemplateResponse>(
      '/tbd.finance.v1.FinanceService/UpsertMailTemplate',
      ($0.UpsertMailTemplateRequest value) => value.writeToBuffer(),
      $0.UpsertMailTemplateResponse.fromBuffer);
  static final _$deleteMailTemplate = $grpc.ClientMethod<
          $0.DeleteMailTemplateRequest, $0.DeleteMailTemplateResponse>(
      '/tbd.finance.v1.FinanceService/DeleteMailTemplate',
      ($0.DeleteMailTemplateRequest value) => value.writeToBuffer(),
      $0.DeleteMailTemplateResponse.fromBuffer);
  static final _$sendMail =
      $grpc.ClientMethod<$0.SendMailRequest, $0.SendMailResponse>(
          '/tbd.finance.v1.FinanceService/SendMail',
          ($0.SendMailRequest value) => value.writeToBuffer(),
          $0.SendMailResponse.fromBuffer);
  static final _$listMail =
      $grpc.ClientMethod<$0.ListMailRequest, $0.ListMailResponse>(
          '/tbd.finance.v1.FinanceService/ListMail',
          ($0.ListMailRequest value) => value.writeToBuffer(),
          $0.ListMailResponse.fromBuffer);
  static final _$getMail =
      $grpc.ClientMethod<$0.GetMailRequest, $0.GetMailResponse>(
          '/tbd.finance.v1.FinanceService/GetMail',
          ($0.GetMailRequest value) => value.writeToBuffer(),
          $0.GetMailResponse.fromBuffer);
  static final _$listDocuments =
      $grpc.ClientMethod<$0.ListDocumentsRequest, $0.ListDocumentsResponse>(
          '/tbd.finance.v1.FinanceService/ListDocuments',
          ($0.ListDocumentsRequest value) => value.writeToBuffer(),
          $0.ListDocumentsResponse.fromBuffer);
  static final _$monthlyReconciliation = $grpc.ClientMethod<
          $0.MonthlyReconciliationRequest, $0.MonthlyReconciliationResponse>(
      '/tbd.finance.v1.FinanceService/MonthlyReconciliation',
      ($0.MonthlyReconciliationRequest value) => value.writeToBuffer(),
      $0.MonthlyReconciliationResponse.fromBuffer);
  static final _$linkDocument =
      $grpc.ClientMethod<$0.LinkDocumentRequest, $0.LinkDocumentResponse>(
          '/tbd.finance.v1.FinanceService/LinkDocument',
          ($0.LinkDocumentRequest value) => value.writeToBuffer(),
          $0.LinkDocumentResponse.fromBuffer);
  static final _$unlinkDocument =
      $grpc.ClientMethod<$0.UnlinkDocumentRequest, $0.UnlinkDocumentResponse>(
          '/tbd.finance.v1.FinanceService/UnlinkDocument',
          ($0.UnlinkDocumentRequest value) => value.writeToBuffer(),
          $0.UnlinkDocumentResponse.fromBuffer);
  static final _$setTransactionNote = $grpc.ClientMethod<
          $0.SetTransactionNoteRequest, $0.SetTransactionNoteResponse>(
      '/tbd.finance.v1.FinanceService/SetTransactionNote',
      ($0.SetTransactionNoteRequest value) => value.writeToBuffer(),
      $0.SetTransactionNoteResponse.fromBuffer);
  static final _$setCounterpartyPolicy = $grpc.ClientMethod<
          $0.SetCounterpartyPolicyRequest, $0.SetCounterpartyPolicyResponse>(
      '/tbd.finance.v1.FinanceService/SetCounterpartyPolicy',
      ($0.SetCounterpartyPolicyRequest value) => value.writeToBuffer(),
      $0.SetCounterpartyPolicyResponse.fromBuffer);
  static final _$deleteCounterpartyPolicy = $grpc.ClientMethod<
          $0.DeleteCounterpartyPolicyRequest,
          $0.DeleteCounterpartyPolicyResponse>(
      '/tbd.finance.v1.FinanceService/DeleteCounterpartyPolicy',
      ($0.DeleteCounterpartyPolicyRequest value) => value.writeToBuffer(),
      $0.DeleteCounterpartyPolicyResponse.fromBuffer);
  static final _$updateDocument =
      $grpc.ClientMethod<$0.UpdateDocumentRequest, $0.UpdateDocumentResponse>(
          '/tbd.finance.v1.FinanceService/UpdateDocument',
          ($0.UpdateDocumentRequest value) => value.writeToBuffer(),
          $0.UpdateDocumentResponse.fromBuffer);
  static final _$extractDocument =
      $grpc.ClientMethod<$0.ExtractDocumentRequest, $0.ExtractDocumentResponse>(
          '/tbd.finance.v1.FinanceService/ExtractDocument',
          ($0.ExtractDocumentRequest value) => value.writeToBuffer(),
          $0.ExtractDocumentResponse.fromBuffer);
  static final _$getDocument =
      $grpc.ClientMethod<$0.GetDocumentRequest, $0.GetDocumentResponse>(
          '/tbd.finance.v1.FinanceService/GetDocument',
          ($0.GetDocumentRequest value) => value.writeToBuffer(),
          $0.GetDocumentResponse.fromBuffer);
  static final _$uploadDocument =
      $grpc.ClientMethod<$0.UploadDocumentRequest, $0.UploadDocumentResponse>(
          '/tbd.finance.v1.FinanceService/UploadDocument',
          ($0.UploadDocumentRequest value) => value.writeToBuffer(),
          $0.UploadDocumentResponse.fromBuffer);
}

@$pb.GrpcServiceName('tbd.finance.v1.FinanceService')
abstract class FinanceServiceBase extends $grpc.Service {
  $core.String get $name => 'tbd.finance.v1.FinanceService';

  FinanceServiceBase() {
    $addMethod($grpc.ServiceMethod<$0.PingRequest, $0.PingResponse>(
        'Ping',
        ping_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.PingRequest.fromBuffer(value),
        ($0.PingResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListTransactionsRequest,
            $0.ListTransactionsResponse>(
        'ListTransactions',
        listTransactions_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListTransactionsRequest.fromBuffer(value),
        ($0.ListTransactionsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetTransactionRequest,
            $0.GetTransactionResponse>(
        'GetTransaction',
        getTransaction_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetTransactionRequest.fromBuffer(value),
        ($0.GetTransactionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.MonthlySummaryRequest,
            $0.MonthlySummaryResponse>(
        'MonthlySummary',
        monthlySummary_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.MonthlySummaryRequest.fromBuffer(value),
        ($0.MonthlySummaryResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListPartiesRequest, $0.ListPartiesResponse>(
            'ListParties',
            listParties_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListPartiesRequest.fromBuffer(value),
            ($0.ListPartiesResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListAccountsRequest, $0.ListAccountsResponse>(
            'ListAccounts',
            listAccounts_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListAccountsRequest.fromBuffer(value),
            ($0.ListAccountsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.RefreshAccountRequest,
            $0.RefreshAccountResponse>(
        'RefreshAccount',
        refreshAccount_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.RefreshAccountRequest.fromBuffer(value),
        ($0.RefreshAccountResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SetAccountSyncRequest,
            $0.SetAccountSyncResponse>(
        'SetAccountSync',
        setAccountSync_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.SetAccountSyncRequest.fromBuffer(value),
        ($0.SetAccountSyncResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListCategoriesRequest,
            $0.ListCategoriesResponse>(
        'ListCategories',
        listCategories_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListCategoriesRequest.fromBuffer(value),
        ($0.ListCategoriesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeclareCategoryRequest,
            $0.DeclareCategoryResponse>(
        'DeclareCategory',
        declareCategory_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeclareCategoryRequest.fromBuffer(value),
        ($0.DeclareCategoryResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpsertCategoryRequest,
            $0.UpsertCategoryResponse>(
        'UpsertCategory',
        upsertCategory_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpsertCategoryRequest.fromBuffer(value),
        ($0.UpsertCategoryResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListRulesRequest, $0.ListRulesResponse>(
        'ListRules',
        listRules_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListRulesRequest.fromBuffer(value),
        ($0.ListRulesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpsertRuleRequest, $0.UpsertRuleResponse>(
        'UpsertRule',
        upsertRule_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.UpsertRuleRequest.fromBuffer(value),
        ($0.UpsertRuleResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.StartConnectionRequest,
            $0.StartConnectionResponse>(
        'StartConnection',
        startConnection_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.StartConnectionRequest.fromBuffer(value),
        ($0.StartConnectionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CompleteConnectionRequest,
            $0.CompleteConnectionResponse>(
        'CompleteConnection',
        completeConnection_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CompleteConnectionRequest.fromBuffer(value),
        ($0.CompleteConnectionResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListConnectionsRequest,
            $0.ListConnectionsResponse>(
        'ListConnections',
        listConnections_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListConnectionsRequest.fromBuffer(value),
        ($0.ListConnectionsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetIssuerRequest, $0.GetIssuerResponse>(
        'GetIssuer',
        getIssuer_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetIssuerRequest.fromBuffer(value),
        ($0.GetIssuerResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.UpsertIssuerRequest, $0.UpsertIssuerResponse>(
            'UpsertIssuer',
            upsertIssuer_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.UpsertIssuerRequest.fromBuffer(value),
            ($0.UpsertIssuerResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListClientsRequest, $0.ListClientsResponse>(
            'ListClients',
            listClients_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListClientsRequest.fromBuffer(value),
            ($0.ListClientsResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.UpsertClientRequest, $0.UpsertClientResponse>(
            'UpsertClient',
            upsertClient_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.UpsertClientRequest.fromBuffer(value),
            ($0.UpsertClientResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListInvoicesRequest, $0.ListInvoicesResponse>(
            'ListInvoices',
            listInvoices_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListInvoicesRequest.fromBuffer(value),
            ($0.ListInvoicesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetInvoiceRequest, $0.GetInvoiceResponse>(
        'GetInvoice',
        getInvoice_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetInvoiceRequest.fromBuffer(value),
        ($0.GetInvoiceResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.CreateInvoiceRequest, $0.CreateInvoiceResponse>(
            'CreateInvoice',
            createInvoice_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CreateInvoiceRequest.fromBuffer(value),
            ($0.CreateInvoiceResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.UpdateInvoiceRequest, $0.UpdateInvoiceResponse>(
            'UpdateInvoice',
            updateInvoice_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.UpdateInvoiceRequest.fromBuffer(value),
            ($0.UpdateInvoiceResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.PreviewInvoiceRequest,
            $0.PreviewInvoiceResponse>(
        'PreviewInvoice',
        previewInvoice_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.PreviewInvoiceRequest.fromBuffer(value),
        ($0.PreviewInvoiceResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ApproveInvoiceRequest,
            $0.ApproveInvoiceResponse>(
        'ApproveInvoice',
        approveInvoice_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ApproveInvoiceRequest.fromBuffer(value),
        ($0.ApproveInvoiceResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.CancelInvoiceRequest, $0.CancelInvoiceResponse>(
            'CancelInvoice',
            cancelInvoice_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.CancelInvoiceRequest.fromBuffer(value),
            ($0.CancelInvoiceResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetInvoiceDocumentRequest,
            $0.GetInvoiceDocumentResponse>(
        'GetInvoiceDocument',
        getInvoiceDocument_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.GetInvoiceDocumentRequest.fromBuffer(value),
        ($0.GetInvoiceDocumentResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListLineTemplatesRequest,
            $0.ListLineTemplatesResponse>(
        'ListLineTemplates',
        listLineTemplates_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListLineTemplatesRequest.fromBuffer(value),
        ($0.ListLineTemplatesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpsertLineTemplateRequest,
            $0.UpsertLineTemplateResponse>(
        'UpsertLineTemplate',
        upsertLineTemplate_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpsertLineTemplateRequest.fromBuffer(value),
        ($0.UpsertLineTemplateResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteLineTemplateRequest,
            $0.DeleteLineTemplateResponse>(
        'DeleteLineTemplate',
        deleteLineTemplate_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteLineTemplateRequest.fromBuffer(value),
        ($0.DeleteLineTemplateResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListConnectorKindsRequest,
            $0.ListConnectorKindsResponse>(
        'ListConnectorKinds',
        listConnectorKinds_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListConnectorKindsRequest.fromBuffer(value),
        ($0.ListConnectorKindsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListConnectorsRequest,
            $0.ListConnectorsResponse>(
        'ListConnectors',
        listConnectors_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListConnectorsRequest.fromBuffer(value),
        ($0.ListConnectorsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.WatchConnectorsRequest,
            $0.WatchConnectorsResponse>(
        'WatchConnectors',
        watchConnectors_Pre,
        false,
        true,
        ($core.List<$core.int> value) =>
            $0.WatchConnectorsRequest.fromBuffer(value),
        ($0.WatchConnectorsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.StartConnectorRequest,
            $0.StartConnectorResponse>(
        'StartConnector',
        startConnector_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.StartConnectorRequest.fromBuffer(value),
        ($0.StartConnectorResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.CompleteConnectorRequest,
            $0.CompleteConnectorResponse>(
        'CompleteConnector',
        completeConnector_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.CompleteConnectorRequest.fromBuffer(value),
        ($0.CompleteConnectorResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.TestConnectorRequest, $0.TestConnectorResponse>(
            'TestConnector',
            testConnector_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.TestConnectorRequest.fromBuffer(value),
            ($0.TestConnectorResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.SyncConnectorRequest, $0.SyncConnectorResponse>(
            'SyncConnector',
            syncConnector_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.SyncConnectorRequest.fromBuffer(value),
            ($0.SyncConnectorResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ConfigureConnectorRequest,
            $0.ConfigureConnectorResponse>(
        'ConfigureConnector',
        configureConnector_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ConfigureConnectorRequest.fromBuffer(value),
        ($0.ConfigureConnectorResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteConnectorRequest,
            $0.DeleteConnectorResponse>(
        'DeleteConnector',
        deleteConnector_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteConnectorRequest.fromBuffer(value),
        ($0.DeleteConnectorResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListConnectorRunsRequest,
            $0.ListConnectorRunsResponse>(
        'ListConnectorRuns',
        listConnectorRuns_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListConnectorRunsRequest.fromBuffer(value),
        ($0.ListConnectorRunsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListMailTemplatesRequest,
            $0.ListMailTemplatesResponse>(
        'ListMailTemplates',
        listMailTemplates_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ListMailTemplatesRequest.fromBuffer(value),
        ($0.ListMailTemplatesResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpsertMailTemplateRequest,
            $0.UpsertMailTemplateResponse>(
        'UpsertMailTemplate',
        upsertMailTemplate_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpsertMailTemplateRequest.fromBuffer(value),
        ($0.UpsertMailTemplateResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteMailTemplateRequest,
            $0.DeleteMailTemplateResponse>(
        'DeleteMailTemplate',
        deleteMailTemplate_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteMailTemplateRequest.fromBuffer(value),
        ($0.DeleteMailTemplateResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SendMailRequest, $0.SendMailResponse>(
        'SendMail',
        sendMail_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.SendMailRequest.fromBuffer(value),
        ($0.SendMailResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ListMailRequest, $0.ListMailResponse>(
        'ListMail',
        listMail_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.ListMailRequest.fromBuffer(value),
        ($0.ListMailResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.GetMailRequest, $0.GetMailResponse>(
        'GetMail',
        getMail_Pre,
        false,
        false,
        ($core.List<$core.int> value) => $0.GetMailRequest.fromBuffer(value),
        ($0.GetMailResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.ListDocumentsRequest, $0.ListDocumentsResponse>(
            'ListDocuments',
            listDocuments_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.ListDocumentsRequest.fromBuffer(value),
            ($0.ListDocumentsResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.MonthlyReconciliationRequest,
            $0.MonthlyReconciliationResponse>(
        'MonthlyReconciliation',
        monthlyReconciliation_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.MonthlyReconciliationRequest.fromBuffer(value),
        ($0.MonthlyReconciliationResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.LinkDocumentRequest, $0.LinkDocumentResponse>(
            'LinkDocument',
            linkDocument_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.LinkDocumentRequest.fromBuffer(value),
            ($0.LinkDocumentResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UnlinkDocumentRequest,
            $0.UnlinkDocumentResponse>(
        'UnlinkDocument',
        unlinkDocument_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UnlinkDocumentRequest.fromBuffer(value),
        ($0.UnlinkDocumentResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SetTransactionNoteRequest,
            $0.SetTransactionNoteResponse>(
        'SetTransactionNote',
        setTransactionNote_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.SetTransactionNoteRequest.fromBuffer(value),
        ($0.SetTransactionNoteResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.SetCounterpartyPolicyRequest,
            $0.SetCounterpartyPolicyResponse>(
        'SetCounterpartyPolicy',
        setCounterpartyPolicy_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.SetCounterpartyPolicyRequest.fromBuffer(value),
        ($0.SetCounterpartyPolicyResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.DeleteCounterpartyPolicyRequest,
            $0.DeleteCounterpartyPolicyResponse>(
        'DeleteCounterpartyPolicy',
        deleteCounterpartyPolicy_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.DeleteCounterpartyPolicyRequest.fromBuffer(value),
        ($0.DeleteCounterpartyPolicyResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UpdateDocumentRequest,
            $0.UpdateDocumentResponse>(
        'UpdateDocument',
        updateDocument_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UpdateDocumentRequest.fromBuffer(value),
        ($0.UpdateDocumentResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.ExtractDocumentRequest,
            $0.ExtractDocumentResponse>(
        'ExtractDocument',
        extractDocument_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.ExtractDocumentRequest.fromBuffer(value),
        ($0.ExtractDocumentResponse value) => value.writeToBuffer()));
    $addMethod(
        $grpc.ServiceMethod<$0.GetDocumentRequest, $0.GetDocumentResponse>(
            'GetDocument',
            getDocument_Pre,
            false,
            false,
            ($core.List<$core.int> value) =>
                $0.GetDocumentRequest.fromBuffer(value),
            ($0.GetDocumentResponse value) => value.writeToBuffer()));
    $addMethod($grpc.ServiceMethod<$0.UploadDocumentRequest,
            $0.UploadDocumentResponse>(
        'UploadDocument',
        uploadDocument_Pre,
        false,
        false,
        ($core.List<$core.int> value) =>
            $0.UploadDocumentRequest.fromBuffer(value),
        ($0.UploadDocumentResponse value) => value.writeToBuffer()));
  }

  $async.Future<$0.PingResponse> ping_Pre(
      $grpc.ServiceCall $call, $async.Future<$0.PingRequest> $request) async {
    return ping($call, await $request);
  }

  $async.Future<$0.PingResponse> ping(
      $grpc.ServiceCall call, $0.PingRequest request);

  $async.Future<$0.ListTransactionsResponse> listTransactions_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListTransactionsRequest> $request) async {
    return listTransactions($call, await $request);
  }

  $async.Future<$0.ListTransactionsResponse> listTransactions(
      $grpc.ServiceCall call, $0.ListTransactionsRequest request);

  $async.Future<$0.GetTransactionResponse> getTransaction_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetTransactionRequest> $request) async {
    return getTransaction($call, await $request);
  }

  $async.Future<$0.GetTransactionResponse> getTransaction(
      $grpc.ServiceCall call, $0.GetTransactionRequest request);

  $async.Future<$0.MonthlySummaryResponse> monthlySummary_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.MonthlySummaryRequest> $request) async {
    return monthlySummary($call, await $request);
  }

  $async.Future<$0.MonthlySummaryResponse> monthlySummary(
      $grpc.ServiceCall call, $0.MonthlySummaryRequest request);

  $async.Future<$0.ListPartiesResponse> listParties_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListPartiesRequest> $request) async {
    return listParties($call, await $request);
  }

  $async.Future<$0.ListPartiesResponse> listParties(
      $grpc.ServiceCall call, $0.ListPartiesRequest request);

  $async.Future<$0.ListAccountsResponse> listAccounts_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListAccountsRequest> $request) async {
    return listAccounts($call, await $request);
  }

  $async.Future<$0.ListAccountsResponse> listAccounts(
      $grpc.ServiceCall call, $0.ListAccountsRequest request);

  $async.Future<$0.RefreshAccountResponse> refreshAccount_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.RefreshAccountRequest> $request) async {
    return refreshAccount($call, await $request);
  }

  $async.Future<$0.RefreshAccountResponse> refreshAccount(
      $grpc.ServiceCall call, $0.RefreshAccountRequest request);

  $async.Future<$0.SetAccountSyncResponse> setAccountSync_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.SetAccountSyncRequest> $request) async {
    return setAccountSync($call, await $request);
  }

  $async.Future<$0.SetAccountSyncResponse> setAccountSync(
      $grpc.ServiceCall call, $0.SetAccountSyncRequest request);

  $async.Future<$0.ListCategoriesResponse> listCategories_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListCategoriesRequest> $request) async {
    return listCategories($call, await $request);
  }

  $async.Future<$0.ListCategoriesResponse> listCategories(
      $grpc.ServiceCall call, $0.ListCategoriesRequest request);

  $async.Future<$0.DeclareCategoryResponse> declareCategory_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeclareCategoryRequest> $request) async {
    return declareCategory($call, await $request);
  }

  $async.Future<$0.DeclareCategoryResponse> declareCategory(
      $grpc.ServiceCall call, $0.DeclareCategoryRequest request);

  $async.Future<$0.UpsertCategoryResponse> upsertCategory_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpsertCategoryRequest> $request) async {
    return upsertCategory($call, await $request);
  }

  $async.Future<$0.UpsertCategoryResponse> upsertCategory(
      $grpc.ServiceCall call, $0.UpsertCategoryRequest request);

  $async.Future<$0.ListRulesResponse> listRules_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListRulesRequest> $request) async {
    return listRules($call, await $request);
  }

  $async.Future<$0.ListRulesResponse> listRules(
      $grpc.ServiceCall call, $0.ListRulesRequest request);

  $async.Future<$0.UpsertRuleResponse> upsertRule_Pre($grpc.ServiceCall $call,
      $async.Future<$0.UpsertRuleRequest> $request) async {
    return upsertRule($call, await $request);
  }

  $async.Future<$0.UpsertRuleResponse> upsertRule(
      $grpc.ServiceCall call, $0.UpsertRuleRequest request);

  $async.Future<$0.StartConnectionResponse> startConnection_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.StartConnectionRequest> $request) async {
    return startConnection($call, await $request);
  }

  $async.Future<$0.StartConnectionResponse> startConnection(
      $grpc.ServiceCall call, $0.StartConnectionRequest request);

  $async.Future<$0.CompleteConnectionResponse> completeConnection_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CompleteConnectionRequest> $request) async {
    return completeConnection($call, await $request);
  }

  $async.Future<$0.CompleteConnectionResponse> completeConnection(
      $grpc.ServiceCall call, $0.CompleteConnectionRequest request);

  $async.Future<$0.ListConnectionsResponse> listConnections_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListConnectionsRequest> $request) async {
    return listConnections($call, await $request);
  }

  $async.Future<$0.ListConnectionsResponse> listConnections(
      $grpc.ServiceCall call, $0.ListConnectionsRequest request);

  $async.Future<$0.GetIssuerResponse> getIssuer_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetIssuerRequest> $request) async {
    return getIssuer($call, await $request);
  }

  $async.Future<$0.GetIssuerResponse> getIssuer(
      $grpc.ServiceCall call, $0.GetIssuerRequest request);

  $async.Future<$0.UpsertIssuerResponse> upsertIssuer_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpsertIssuerRequest> $request) async {
    return upsertIssuer($call, await $request);
  }

  $async.Future<$0.UpsertIssuerResponse> upsertIssuer(
      $grpc.ServiceCall call, $0.UpsertIssuerRequest request);

  $async.Future<$0.ListClientsResponse> listClients_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListClientsRequest> $request) async {
    return listClients($call, await $request);
  }

  $async.Future<$0.ListClientsResponse> listClients(
      $grpc.ServiceCall call, $0.ListClientsRequest request);

  $async.Future<$0.UpsertClientResponse> upsertClient_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpsertClientRequest> $request) async {
    return upsertClient($call, await $request);
  }

  $async.Future<$0.UpsertClientResponse> upsertClient(
      $grpc.ServiceCall call, $0.UpsertClientRequest request);

  $async.Future<$0.ListInvoicesResponse> listInvoices_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListInvoicesRequest> $request) async {
    return listInvoices($call, await $request);
  }

  $async.Future<$0.ListInvoicesResponse> listInvoices(
      $grpc.ServiceCall call, $0.ListInvoicesRequest request);

  $async.Future<$0.GetInvoiceResponse> getInvoice_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetInvoiceRequest> $request) async {
    return getInvoice($call, await $request);
  }

  $async.Future<$0.GetInvoiceResponse> getInvoice(
      $grpc.ServiceCall call, $0.GetInvoiceRequest request);

  $async.Future<$0.CreateInvoiceResponse> createInvoice_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CreateInvoiceRequest> $request) async {
    return createInvoice($call, await $request);
  }

  $async.Future<$0.CreateInvoiceResponse> createInvoice(
      $grpc.ServiceCall call, $0.CreateInvoiceRequest request);

  $async.Future<$0.UpdateInvoiceResponse> updateInvoice_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateInvoiceRequest> $request) async {
    return updateInvoice($call, await $request);
  }

  $async.Future<$0.UpdateInvoiceResponse> updateInvoice(
      $grpc.ServiceCall call, $0.UpdateInvoiceRequest request);

  $async.Future<$0.PreviewInvoiceResponse> previewInvoice_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.PreviewInvoiceRequest> $request) async {
    return previewInvoice($call, await $request);
  }

  $async.Future<$0.PreviewInvoiceResponse> previewInvoice(
      $grpc.ServiceCall call, $0.PreviewInvoiceRequest request);

  $async.Future<$0.ApproveInvoiceResponse> approveInvoice_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ApproveInvoiceRequest> $request) async {
    return approveInvoice($call, await $request);
  }

  $async.Future<$0.ApproveInvoiceResponse> approveInvoice(
      $grpc.ServiceCall call, $0.ApproveInvoiceRequest request);

  $async.Future<$0.CancelInvoiceResponse> cancelInvoice_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CancelInvoiceRequest> $request) async {
    return cancelInvoice($call, await $request);
  }

  $async.Future<$0.CancelInvoiceResponse> cancelInvoice(
      $grpc.ServiceCall call, $0.CancelInvoiceRequest request);

  $async.Future<$0.GetInvoiceDocumentResponse> getInvoiceDocument_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.GetInvoiceDocumentRequest> $request) async {
    return getInvoiceDocument($call, await $request);
  }

  $async.Future<$0.GetInvoiceDocumentResponse> getInvoiceDocument(
      $grpc.ServiceCall call, $0.GetInvoiceDocumentRequest request);

  $async.Future<$0.ListLineTemplatesResponse> listLineTemplates_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListLineTemplatesRequest> $request) async {
    return listLineTemplates($call, await $request);
  }

  $async.Future<$0.ListLineTemplatesResponse> listLineTemplates(
      $grpc.ServiceCall call, $0.ListLineTemplatesRequest request);

  $async.Future<$0.UpsertLineTemplateResponse> upsertLineTemplate_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpsertLineTemplateRequest> $request) async {
    return upsertLineTemplate($call, await $request);
  }

  $async.Future<$0.UpsertLineTemplateResponse> upsertLineTemplate(
      $grpc.ServiceCall call, $0.UpsertLineTemplateRequest request);

  $async.Future<$0.DeleteLineTemplateResponse> deleteLineTemplate_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteLineTemplateRequest> $request) async {
    return deleteLineTemplate($call, await $request);
  }

  $async.Future<$0.DeleteLineTemplateResponse> deleteLineTemplate(
      $grpc.ServiceCall call, $0.DeleteLineTemplateRequest request);

  $async.Future<$0.ListConnectorKindsResponse> listConnectorKinds_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListConnectorKindsRequest> $request) async {
    return listConnectorKinds($call, await $request);
  }

  $async.Future<$0.ListConnectorKindsResponse> listConnectorKinds(
      $grpc.ServiceCall call, $0.ListConnectorKindsRequest request);

  $async.Future<$0.ListConnectorsResponse> listConnectors_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListConnectorsRequest> $request) async {
    return listConnectors($call, await $request);
  }

  $async.Future<$0.ListConnectorsResponse> listConnectors(
      $grpc.ServiceCall call, $0.ListConnectorsRequest request);

  $async.Stream<$0.WatchConnectorsResponse> watchConnectors_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.WatchConnectorsRequest> $request) async* {
    yield* watchConnectors($call, await $request);
  }

  $async.Stream<$0.WatchConnectorsResponse> watchConnectors(
      $grpc.ServiceCall call, $0.WatchConnectorsRequest request);

  $async.Future<$0.StartConnectorResponse> startConnector_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.StartConnectorRequest> $request) async {
    return startConnector($call, await $request);
  }

  $async.Future<$0.StartConnectorResponse> startConnector(
      $grpc.ServiceCall call, $0.StartConnectorRequest request);

  $async.Future<$0.CompleteConnectorResponse> completeConnector_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.CompleteConnectorRequest> $request) async {
    return completeConnector($call, await $request);
  }

  $async.Future<$0.CompleteConnectorResponse> completeConnector(
      $grpc.ServiceCall call, $0.CompleteConnectorRequest request);

  $async.Future<$0.TestConnectorResponse> testConnector_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.TestConnectorRequest> $request) async {
    return testConnector($call, await $request);
  }

  $async.Future<$0.TestConnectorResponse> testConnector(
      $grpc.ServiceCall call, $0.TestConnectorRequest request);

  $async.Future<$0.SyncConnectorResponse> syncConnector_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.SyncConnectorRequest> $request) async {
    return syncConnector($call, await $request);
  }

  $async.Future<$0.SyncConnectorResponse> syncConnector(
      $grpc.ServiceCall call, $0.SyncConnectorRequest request);

  $async.Future<$0.ConfigureConnectorResponse> configureConnector_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ConfigureConnectorRequest> $request) async {
    return configureConnector($call, await $request);
  }

  $async.Future<$0.ConfigureConnectorResponse> configureConnector(
      $grpc.ServiceCall call, $0.ConfigureConnectorRequest request);

  $async.Future<$0.DeleteConnectorResponse> deleteConnector_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteConnectorRequest> $request) async {
    return deleteConnector($call, await $request);
  }

  $async.Future<$0.DeleteConnectorResponse> deleteConnector(
      $grpc.ServiceCall call, $0.DeleteConnectorRequest request);

  $async.Future<$0.ListConnectorRunsResponse> listConnectorRuns_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListConnectorRunsRequest> $request) async {
    return listConnectorRuns($call, await $request);
  }

  $async.Future<$0.ListConnectorRunsResponse> listConnectorRuns(
      $grpc.ServiceCall call, $0.ListConnectorRunsRequest request);

  $async.Future<$0.ListMailTemplatesResponse> listMailTemplates_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListMailTemplatesRequest> $request) async {
    return listMailTemplates($call, await $request);
  }

  $async.Future<$0.ListMailTemplatesResponse> listMailTemplates(
      $grpc.ServiceCall call, $0.ListMailTemplatesRequest request);

  $async.Future<$0.UpsertMailTemplateResponse> upsertMailTemplate_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpsertMailTemplateRequest> $request) async {
    return upsertMailTemplate($call, await $request);
  }

  $async.Future<$0.UpsertMailTemplateResponse> upsertMailTemplate(
      $grpc.ServiceCall call, $0.UpsertMailTemplateRequest request);

  $async.Future<$0.DeleteMailTemplateResponse> deleteMailTemplate_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.DeleteMailTemplateRequest> $request) async {
    return deleteMailTemplate($call, await $request);
  }

  $async.Future<$0.DeleteMailTemplateResponse> deleteMailTemplate(
      $grpc.ServiceCall call, $0.DeleteMailTemplateRequest request);

  $async.Future<$0.SendMailResponse> sendMail_Pre($grpc.ServiceCall $call,
      $async.Future<$0.SendMailRequest> $request) async {
    return sendMail($call, await $request);
  }

  $async.Future<$0.SendMailResponse> sendMail(
      $grpc.ServiceCall call, $0.SendMailRequest request);

  $async.Future<$0.ListMailResponse> listMail_Pre($grpc.ServiceCall $call,
      $async.Future<$0.ListMailRequest> $request) async {
    return listMail($call, await $request);
  }

  $async.Future<$0.ListMailResponse> listMail(
      $grpc.ServiceCall call, $0.ListMailRequest request);

  $async.Future<$0.GetMailResponse> getMail_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetMailRequest> $request) async {
    return getMail($call, await $request);
  }

  $async.Future<$0.GetMailResponse> getMail(
      $grpc.ServiceCall call, $0.GetMailRequest request);

  $async.Future<$0.ListDocumentsResponse> listDocuments_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ListDocumentsRequest> $request) async {
    return listDocuments($call, await $request);
  }

  $async.Future<$0.ListDocumentsResponse> listDocuments(
      $grpc.ServiceCall call, $0.ListDocumentsRequest request);

  $async.Future<$0.MonthlyReconciliationResponse> monthlyReconciliation_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.MonthlyReconciliationRequest> $request) async {
    return monthlyReconciliation($call, await $request);
  }

  $async.Future<$0.MonthlyReconciliationResponse> monthlyReconciliation(
      $grpc.ServiceCall call, $0.MonthlyReconciliationRequest request);

  $async.Future<$0.LinkDocumentResponse> linkDocument_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.LinkDocumentRequest> $request) async {
    return linkDocument($call, await $request);
  }

  $async.Future<$0.LinkDocumentResponse> linkDocument(
      $grpc.ServiceCall call, $0.LinkDocumentRequest request);

  $async.Future<$0.UnlinkDocumentResponse> unlinkDocument_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UnlinkDocumentRequest> $request) async {
    return unlinkDocument($call, await $request);
  }

  $async.Future<$0.UnlinkDocumentResponse> unlinkDocument(
      $grpc.ServiceCall call, $0.UnlinkDocumentRequest request);

  $async.Future<$0.SetTransactionNoteResponse> setTransactionNote_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.SetTransactionNoteRequest> $request) async {
    return setTransactionNote($call, await $request);
  }

  $async.Future<$0.SetTransactionNoteResponse> setTransactionNote(
      $grpc.ServiceCall call, $0.SetTransactionNoteRequest request);

  $async.Future<$0.SetCounterpartyPolicyResponse> setCounterpartyPolicy_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.SetCounterpartyPolicyRequest> $request) async {
    return setCounterpartyPolicy($call, await $request);
  }

  $async.Future<$0.SetCounterpartyPolicyResponse> setCounterpartyPolicy(
      $grpc.ServiceCall call, $0.SetCounterpartyPolicyRequest request);

  $async.Future<$0.DeleteCounterpartyPolicyResponse>
      deleteCounterpartyPolicy_Pre($grpc.ServiceCall $call,
          $async.Future<$0.DeleteCounterpartyPolicyRequest> $request) async {
    return deleteCounterpartyPolicy($call, await $request);
  }

  $async.Future<$0.DeleteCounterpartyPolicyResponse> deleteCounterpartyPolicy(
      $grpc.ServiceCall call, $0.DeleteCounterpartyPolicyRequest request);

  $async.Future<$0.UpdateDocumentResponse> updateDocument_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UpdateDocumentRequest> $request) async {
    return updateDocument($call, await $request);
  }

  $async.Future<$0.UpdateDocumentResponse> updateDocument(
      $grpc.ServiceCall call, $0.UpdateDocumentRequest request);

  $async.Future<$0.ExtractDocumentResponse> extractDocument_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.ExtractDocumentRequest> $request) async {
    return extractDocument($call, await $request);
  }

  $async.Future<$0.ExtractDocumentResponse> extractDocument(
      $grpc.ServiceCall call, $0.ExtractDocumentRequest request);

  $async.Future<$0.GetDocumentResponse> getDocument_Pre($grpc.ServiceCall $call,
      $async.Future<$0.GetDocumentRequest> $request) async {
    return getDocument($call, await $request);
  }

  $async.Future<$0.GetDocumentResponse> getDocument(
      $grpc.ServiceCall call, $0.GetDocumentRequest request);

  $async.Future<$0.UploadDocumentResponse> uploadDocument_Pre(
      $grpc.ServiceCall $call,
      $async.Future<$0.UploadDocumentRequest> $request) async {
    return uploadDocument($call, await $request);
  }

  $async.Future<$0.UploadDocumentResponse> uploadDocument(
      $grpc.ServiceCall call, $0.UploadDocumentRequest request);
}
