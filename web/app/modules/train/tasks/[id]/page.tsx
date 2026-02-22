import { TrainTaskDetail } from "@/components/train/train-task-detail";
import { requireUser } from "@/lib/server-auth";

export default async function TrainTaskDetailPage({ params }: { params: Promise<{ id: string }> }) {
  await requireUser();
  const resolvedParams = await params;
  return <TrainTaskDetail taskId={resolvedParams.id} />;
}
