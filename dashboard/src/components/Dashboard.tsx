import { useState, useEffect, useRef } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Download, Search, Activity, Database, CheckCircle, Clock } from "lucide-react";
import { ProofTable } from "./ProofTable";
import { StatsCards } from "./StatsCards";

export function Dashboard() {
  const [searchTerm, setSearchTerm] = useState("");
  const [data, setData] = useState([]);
  const [filteredData, setFilteredData] = useState([]);
  const intervalRef = useRef<NodeJS.Timeout | null>(null);

  // Fetch data from API
  const fetchData = async () => {
    try {
      const res = await fetch("/api/epochs", {
        headers: {
          "Accept": "application/json",
          "ngrok-skip-browser-warning": "true"
        }
      });
      if (!res.ok) throw new Error("Failed to fetch data");
      const json = await res.json();
      setData(json);
      // If no search term, update filteredData as well
      if (!searchTerm) setFilteredData(json);
      else handleSearch(searchTerm, json);
    } catch (err) {
      // Optionally handle error 
      // console.error(err);
    }
  };

  useEffect(() => {
    fetchData();
    intervalRef.current = setInterval(fetchData, 10000); // 10 seconds
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);

  // Search handler
  const handleSearch = (value: string, baseData = data) => {
    setSearchTerm(value);
    if (!value) {
      setFilteredData(baseData);
      return;
    }
    const filtered = baseData.filter((item: any) =>
      item.uuid.toLowerCase().includes(value.toLowerCase()) ||
      item.epoch_number.toString().includes(value) ||
      item.slot_number.toString().includes(value) ||
      (item.proof_id !== null && item.proof_id.toString().includes(value)) ||
      (item.status && item.status.toLowerCase().includes(value.toLowerCase()))
    );
    setFilteredData(filtered);
  };

  return (
    <div className="min-h-screen bg-[#0A191E] text-slate-200">
      <div className="container mx-auto p-6 space-y-8">
        {/* Header */}
        <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
          <div className="flex items-center gap-4">
            <img src="/logo.svg" alt="Logo" className="h-10 w-10" />
            <div>
              <h1 className="text-4xl font-bold text-white">
                Bankai Dashboard
              </h1>
              <p className="text-slate-400 mt-1">
                Explore the Bankai Proofs
              </p>
            </div>
          </div>
          
          {/* Search */}
          <div className="relative w-full sm:w-80">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-slate-400 h-4 w-4" />
            <Input
              placeholder="Search by UUID, epoch, slot, or proof ID..."
              value={searchTerm}
              onChange={(e) => handleSearch(e.target.value)}
              className="pl-10 bg-[#112229] border-slate-700 text-white placeholder-slate-400"
            />
          </div>
        </div>

        {/* Stats Cards */}
        <StatsCards data={filteredData} />

        {/* Main Data Table */}
        <Card className="shadow-2xl bg-[#112229] border-slate-800">
          <CardHeader className="bg-slate-900/40 border-b border-slate-800 rounded-t-lg">
            <CardTitle className="flex items-center gap-3 text-white">
              <Database className="h-5 w-5 text-accent" />
              Proof Records
              <Badge variant="secondary" className="ml-auto bg-slate-700 text-slate-300">
                {filteredData.length} records
              </Badge>
            </CardTitle>
          </CardHeader>
          <CardContent className="p-0">
            <ProofTable data={filteredData} />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}
