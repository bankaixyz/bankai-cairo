import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Activity, CheckCircle, Clock, Database } from "lucide-react";

interface StatsCardsProps {
  data: any[];
}

export function StatsCards({ data }: StatsCardsProps) {
  const totalProofs = data.length;
  const completedProofs = data.filter(item => item.status === "done").length;
  const averageSigners = Math.round(data.reduce((acc, item) => acc + (item.outputs?.n_signers || 0), 0) / data.length) || 0;
  const latestEpoch = data.length > 0 ? Math.max(...data.map(item => item.epoch_number)) : 0;

  const stats = [
    {
      title: "Total Proofs",
      value: totalProofs,
      icon: Database,
      color: "text-blue-400",
    },
    {
      title: "Completed",
      value: completedProofs,
      icon: CheckCircle,
      color: "text-green-400",
    },
    {
      title: "Avg Signers",
      value: averageSigners,
      icon: Activity,
      color: "text-purple-400",
    },
    {
      title: "Latest Epoch",
      value: latestEpoch,
      icon: Clock,
      color: "text-orange-400",
    }
  ];

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
      {stats.map((stat, index) => (
        <Card key={index} className="bg-[#112229] border border-slate-800 shadow-lg hover:border-slate-700 transition-colors">
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium text-slate-400">
              {stat.title}
            </CardTitle>
            <stat.icon className={`h-4 w-4 ${stat.color}`} />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-white">{stat.value.toLocaleString()}</div>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
