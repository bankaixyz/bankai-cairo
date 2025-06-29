import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Download,
  Copy,
  Eye,
  EyeOff,
  DatabaseZap,
  Terminal,
  Users,
} from "lucide-react";
import { useToast } from "@/hooks/use-toast";

interface ProofTableProps {
  data: any[];
}

export function ProofTable({ data }: ProofTableProps) {
  const { toast } = useToast();
  const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set());

  const downloadProofOutput = (item: any) => {
    if (!item.proof_id) {
      toast({
        title: "Download Unavailable",
        description: "A proof has not been generated for this epoch yet.",
      });
      return;
    }

    const downloadUrl = `/api/proofs/${item.proof_id}`;
    const a = document.createElement("a");
    a.href = downloadUrl;
    // The browser will use the filename from the server's Content-Disposition header,
    // but we can suggest a fallback.
    a.download = `proof-${item.proof_id}`; 
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    
    toast({
      title: "Download Started",
      description: `Proof for ID ${item.proof_id} is downloading...`,
    });
  };

  const copyToClipboard = (text: string, label: string) => {
    navigator.clipboard.writeText(text);
    toast({
      title: "Copied to Clipboard",
      description: `${label} has been copied to your clipboard.`,
    });
  };

  const truncateHash = (hash: string, length: number = 10) => {
    if (!hash) return "";
    return hash.length > length ? `${hash.slice(0, length)}...${hash.slice(-6)}` : hash;
  };

  const toggleRowExpansion = (uuid: string) => {
    const newExpandedRows = new Set(expandedRows);
    if (newExpandedRows.has(uuid)) {
      newExpandedRows.delete(uuid);
    } else {
      newExpandedRows.add(uuid);
    }
    setExpandedRows(newExpandedRows);
  };

  const getSignerColor = (signerCount: number | null | undefined) => {
    if (signerCount === null || signerCount === undefined) {
      return "text-slate-500";
    }
    const ratio = signerCount / 512;
    if (ratio < 0.66) return "font-bold text-red-400";
    if (ratio < 0.85) return "font-bold text-orange-400";
    return "font-bold text-green-400";
  };

  return (
    <div className="overflow-x-auto">
      <table className="w-full">
        <thead className="border-b border-slate-800">
          <tr>
            <th className="text-left p-4 font-semibold text-slate-400">Actions</th>
            <th className="text-left p-4 font-semibold text-slate-400">Epoch</th>
            <th className="text-left p-4 font-semibold text-slate-400">Slot</th>
            <th className="text-left p-4 font-semibold text-slate-400">Status</th>
            <th className="text-left p-4 font-semibold text-slate-400">Beacon Header Root</th>
            <th className="text-left p-4 font-semibold text-slate-400">Execution Header Hash</th>
            <th className="text-left p-4 font-semibold text-slate-400">Signers</th>
          </tr>
        </thead>
        <tbody>
          {data.map((item, index) => (
            <>
              <tr
                key={item.uuid}
                className="border-b border-slate-800 hover:bg-slate-800/40 transition-colors"
              >
                <td className="p-4">
                  <Button
                    size="sm"
                    variant="outline"
                    onClick={() => toggleRowExpansion(item.uuid)}
                    className="bg-slate-800 text-slate-300 border-slate-700 hover:bg-slate-700 hover:text-white"
                  >
                    {expandedRows.has(item.uuid) ? (
                      <EyeOff className="h-3 w-3" />
                    ) : (
                      <Eye className="h-3 w-3" />
                    )}
                  </Button>
                </td>
                <td className="p-4">
                  <div className="font-mono text-sm font-semibold text-accent">
                    {item.epoch_number.toLocaleString()}
                  </div>
                </td>
                <td className="p-4">
                  <div className="font-mono text-sm text-slate-400">
                    {item.slot_number.toLocaleString()}
                  </div>
                </td>
                <td className="p-4">
                  <Badge
                    variant={item.status === "done" ? "default" : "secondary"}
                    className={
                      item.status === "done"
                        ? "bg-green-500/20 text-green-300 border border-green-500/30"
                        : "bg-yellow-500/20 text-yellow-300 border border-yellow-500/30"
                    }
                  >
                    {item.status}
                  </Badge>
                </td>
                <td className="p-4">
                  <div className="flex items-center gap-2">
                    <code className="bg-slate-800 text-slate-400 px-2 py-1 rounded text-xs font-mono">
                      {truncateHash(item.outputs?.beacon_header_root)}
                    </code>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() =>
                        copyToClipboard(
                          item.outputs?.beacon_header_root,
                          "Beacon Header Root"
                        )
                      }
                      className="h-6 w-6 p-0 text-slate-400 hover:bg-slate-800 hover:text-white"
                    >
                      <Copy className="h-3 w-3" />
                    </Button>
                  </div>
                </td>
                <td className="p-4">
                  <div className="flex items-center gap-2">
                    <code className="bg-slate-800 text-slate-400 px-2 py-1 rounded text-xs font-mono">
                      {truncateHash(item.outputs?.execution_header_root)}
                    </code>
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() =>
                        copyToClipboard(
                          item.outputs?.execution_header_root,
                          "Execution Header Hash"
                        )
                      }
                      className="h-6 w-6 p-0 text-slate-400 hover:bg-slate-800 hover:text-white"
                    >
                      <Copy className="h-3 w-3" />
                    </Button>
                  </div>
                </td>
                <td className="p-4">
                  <div className="flex items-center gap-2">
                    <div className="font-semibold text-green-400">
                      {item.outputs?.n_signers}
                    </div>
                    <div className="text-xs text-slate-500">signers</div>
                  </div>
                </td>
              </tr>
              {expandedRows.has(item.uuid) && (
                <tr className="bg-slate-900/50">
                  <td colSpan={7} className="p-4">
                    <div className="bg-[#0A191E] rounded-lg p-4 shadow-inner border border-slate-800">
                      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                        {/* Beacon Section */}
                        <div className="space-y-4">
                          <h4 className="font-semibold text-white flex items-center gap-2">
                            <DatabaseZap className="h-4 w-4 text-accent" /> Beacon
                          </h4>
                          <div>
                            <span className="font-medium text-slate-400">
                              Beacon Height (Slot):
                            </span>
                            <div className="font-mono text-lg text-slate-300">
                              {item.slot_number?.toLocaleString() ?? "N/A"}
                            </div>
                          </div>
                          <div>
                            <span className="font-medium text-slate-400">
                              Header Root:
                            </span>
                            <div className="font-mono text-xs bg-slate-800 text-slate-300 p-2 rounded mt-1 break-all flex items-center justify-between">
                              <span className="truncate">
                                {item.outputs?.beacon_header_root || "N/A"}
                              </span>
                              <Button
                                size="icon"
                                variant="ghost"
                                onClick={() =>
                                  copyToClipboard(
                                    item.outputs?.beacon_header_root,
                                    "Beacon Header Root"
                                  )
                                }
                                className="h-6 w-6 p-1 flex-shrink-0"
                              >
                                <Copy className="h-3 w-3" />
                              </Button>
                            </div>
                          </div>
                          <div>
                            <span className="font-medium text-slate-400">
                              State Root:
                            </span>
                            <div className="font-mono text-xs bg-slate-800 text-slate-300 p-2 rounded mt-1 break-all flex items-center justify-between">
                              <span className="truncate">
                                {item.outputs?.beacon_state_root || "N/A"}
                              </span>
                              <Button
                                size="icon"
                                variant="ghost"
                                onClick={() =>
                                  copyToClipboard(
                                    item.outputs?.beacon_state_root,
                                    "Beacon State Root"
                                  )
                                }
                                className="h-6 w-6 p-1 flex-shrink-0"
                              >
                                <Copy className="h-3 w-3" />
                              </Button>
                            </div>
                          </div>
                        </div>

                        {/* Execution Section */}
                        <div className="space-y-4">
                          <h4 className="font-semibold text-white flex items-center gap-2">
                            <Terminal className="h-4 w-4 text-accent" />{" "}
                            Execution
                          </h4>
                          <div>
                            <span className="font-medium text-slate-400">
                              Execution Height:
                            </span>
                            <div className="font-mono text-lg text-slate-300">
                              {item.outputs?.execution_header_height?.toLocaleString() ?? "N/A"}
                            </div>
                          </div>
                          <div>
                            <span className="font-medium text-slate-400">
                              Header Hash:
                            </span>
                            <div className="font-mono text-xs bg-slate-800 text-slate-300 p-2 rounded mt-1 break-all flex items-center justify-between">
                              <span className="truncate">
                                {item.outputs?.execution_header_root || "N/A"}
                              </span>
                              <Button
                                size="icon"
                                variant="ghost"
                                onClick={() =>
                                  copyToClipboard(
                                    item.outputs?.execution_header_root,
                                    "Execution Header Hash"
                                  )
                                }
                                className="h-6 w-6 p-1 flex-shrink-0"
                              >
                                <Copy className="h-3 w-3" />
                              </Button>
                            </div>
                          </div>
                        </div>

                        {/* Signing Section */}
                        <div className="space-y-4">
                          <h4 className="font-semibold text-white flex items-center gap-2">
                            <Users className="h-4 w-4 text-accent" /> Signing
                          </h4>
                          <div>
                            <span className="font-medium text-slate-400">
                              Signer's Participation:
                            </span>
                            <div
                              className={`font-mono text-lg ${getSignerColor(
                                item.outputs?.n_signers
                              )}`}
                            >
                              {item.outputs?.n_signers ?? "N/A"} / 512
                            </div>
                          </div>
                          <div>
                            <span className="font-medium text-slate-400">
                              Current Committee Hash:
                            </span>
                            <div className="font-mono text-xs bg-slate-800 text-slate-300 p-2 rounded mt-1 break-all flex items-center justify-between">
                              <span className="truncate">
                                {item.outputs?.current_committee_hash || "N/A"}
                              </span>
                              <Button
                                size="icon"
                                variant="ghost"
                                onClick={() =>
                                  copyToClipboard(
                                    item.outputs?.current_committee_hash,
                                    "Current Committee Hash"
                                  )
                                }
                                className="h-6 w-6 p-1 flex-shrink-0"
                              >
                                <Copy className="h-3 w-3" />
                              </Button>
                            </div>
                          </div>
                          <div>
                            <span className="font-medium text-slate-400">
                              Next Committee Hash:
                            </span>
                            <div className="font-mono text-xs bg-slate-800 text-slate-300 p-2 rounded mt-1 break-all flex items-center justify-between">
                              <span className="truncate">
                                {item.outputs?.next_committee_hash || "N/A"}
                              </span>
                              <Button
                                size="icon"
                                variant="ghost"
                                onClick={() =>
                                  copyToClipboard(
                                    item.outputs?.next_committee_hash,
                                    "Next Committee Hash"
                                  )
                                }
                                className="h-6 w-6 p-1 flex-shrink-0"
                              >
                                <Copy className="h-3 w-3" />
                              </Button>
                            </div>
                          </div>
                        </div>
                      </div>

                      {/* Footer with IDs and Download button */}
                      <div className="mt-6 border-t border-slate-800 pt-4 flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-6 gap-y-2 text-xs w-full">
                          <div>
                            <span className="font-medium text-slate-400">
                              UUID:
                            </span>
                            <div className="font-mono text-slate-500 break-all">
                              {item.uuid}
                            </div>
                          </div>
                          <div>
                            <span className="font-medium text-slate-400">
                              Atlantic ID:
                            </span>
                            <div className="font-mono text-slate-500 break-all">
                              {item.atlantic_id ?? "N/A"}
                            </div>
                          </div>
                        </div>
                        <Button
                          onClick={() => downloadProofOutput(item)}
                          disabled={!item.proof_id}
                          className="bg-accent text-black hover:bg-accent-dark font-bold disabled:opacity-40 disabled:cursor-not-allowed w-full sm:w-auto flex-shrink-0"
                        >
                          <Download className="h-4 w-4 mr-2" />
                          Download Proof
                        </Button>
                      </div>
                    </div>
                  </td>
                </tr>
              )}
            </>
          ))}
        </tbody>
      </table>
    </div>
  );
}
