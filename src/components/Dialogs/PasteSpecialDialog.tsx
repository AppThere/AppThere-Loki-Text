import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
    DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Type, Clipboard, FileText } from "lucide-react";

export type PasteOption = 'as-is' | 'plain' | 'semantic';

interface PasteSpecialDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSelect: (option: PasteOption) => void;
}

export function PasteSpecialDialog({ open, onOpenChange, onSelect }: PasteSpecialDialogProps) {
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-md">
                <DialogHeader>
                    <DialogTitle>Paste Special</DialogTitle>
                    <DialogDescription>
                        This content contains formatting. How would you like to paste it?
                    </DialogDescription>
                </DialogHeader>
                <div className="grid grid-cols-1 gap-4 py-4">
                    <Button
                        variant="outline"
                        className="flex items-center justify-start h-16 px-4 space-x-4 border-2 hover:border-blue-500 hover:bg-blue-50/50 dark:hover:bg-blue-900/20"
                        onClick={() => onSelect('as-is')}
                    >
                        <div className="p-2 bg-blue-100 rounded-full dark:bg-blue-900/40">
                            <Clipboard className="w-5 h-5 text-blue-600 dark:text-blue-400" />
                        </div>
                        <div className="text-left">
                            <div className="font-semibold">Paste as is</div>
                            <div className="text-xs text-muted-foreground text-pretty">Keep all original formatting and styles</div>
                        </div>
                    </Button>

                    <Button
                        variant="outline"
                        className="flex items-center justify-start h-16 px-4 space-x-4 border-2 hover:border-blue-500 hover:bg-blue-50/50 dark:hover:bg-blue-900/20"
                        onClick={() => onSelect('semantic')}
                    >
                        <div className="p-2 bg-green-100 rounded-full dark:bg-green-900/40">
                            <FileText className="w-5 h-5 text-green-600 dark:text-green-400" />
                        </div>
                        <div className="text-left">
                            <div className="font-semibold">Semantic Paste</div>
                            <div className="text-xs text-muted-foreground text-pretty">Keep structure (bold, lists) but strip local fonts/colors</div>
                        </div>
                    </Button>

                    <Button
                        variant="outline"
                        className="flex items-center justify-start h-16 px-4 space-x-4 border-2 hover:border-blue-500 hover:bg-blue-50/50 dark:hover:bg-blue-900/20"
                        onClick={() => onSelect('plain')}
                    >
                        <div className="p-2 bg-slate-100 rounded-full dark:bg-slate-800">
                            <Type className="w-5 h-5 text-slate-600 dark:text-slate-400" />
                        </div>
                        <div className="text-left">
                            <div className="font-semibold">Paste as Plain Text</div>
                            <div className="text-xs text-muted-foreground text-pretty">Remove all formatting and insert text only</div>
                        </div>
                    </Button>
                </div>
                <DialogFooter>
                    <Button variant="ghost" onClick={() => onOpenChange(false)}>
                        Cancel
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
