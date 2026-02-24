import { ApplicationReviewGate } from "@/components/application-review-gate";
import { requirePendingReviewUser } from "@/lib/server-auth";

export default async function ApplicationReviewPage() {
  const user = await requirePendingReviewUser();
  return <ApplicationReviewGate email={user.email} />;
}
