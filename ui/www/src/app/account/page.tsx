import type { Metadata } from "next";

import { AccountContent } from "@/components/pages/account";

export const metadata: Metadata = {
  title: "Account",
  description: "Who is signed in on this site.",
  alternates: { canonical: "/account/" },
  // Behind a sign-in at the gateway: nothing to index.
  robots: { index: false, follow: false },
};

export default function AccountPage() {
  return <AccountContent />;
}
