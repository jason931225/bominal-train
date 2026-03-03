import { requireApprovedUser } from "@/lib/server-auth";
import { TrainDashboard } from "@/components/train/train-dashboard";

export default async function TrainModulePage() {
  await requireApprovedUser();
  return <TrainDashboard />;
}
