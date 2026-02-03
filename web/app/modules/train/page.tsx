import { requireUser } from "@/lib/server-auth";
import { TrainDashboard } from "@/components/train/train-dashboard";

export default async function TrainModulePage() {
  await requireUser();
  return <TrainDashboard />;
}
