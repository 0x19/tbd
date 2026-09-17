import { PageTitle } from "@/components/kit";
import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

export default function InvoicesPage() {
  return (
    <>
      <PageTitle title="Invoices" description="Create, approve and send; every invoice ever issued." />
      <Card className="max-w-xl">
        <CardHeader>
          <CardTitle>Not built yet</CardTitle>
          <CardDescription>
            The invoice workflow is the next phase (docs/plans/finance-service.md, P6). Until then this page
            says so rather than showing a placeholder that looks like data.
          </CardDescription>
        </CardHeader>
      </Card>
    </>
  );
}
