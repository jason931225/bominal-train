import { redirect } from "next/navigation";

export default async function PaymentPage() {
  redirect("/settings/payment");
}
