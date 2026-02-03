import { TrainTaskDetail } from "@/components/train/train-task-detail";
import { requireUser } from "@/lib/server-auth";

export default async function TrainTaskDetailPage({ params }: { params: { id: string } }) {
  await requireUser();
  return <TrainTaskDetail taskId={params.id} />;
}
