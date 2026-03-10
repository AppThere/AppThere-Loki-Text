import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogHeader,
    DialogTitle,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { FileText, FileCode, FileDown } from 'lucide-react';

export type FileType = 'odt' | 'fodt';

interface FileTypeDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSelect: (type: FileType) => void;
}

export function FileTypeDialog({ open, onOpenChange, onSelect }: FileTypeDialogProps) {
    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-md">
                <DialogHeader>
                    <DialogTitle>Choose File Format</DialogTitle>
                    <DialogDescription>
                        Select the format you'd like to use for saving this document.
                    </DialogDescription>
                </DialogHeader>
                <div className="grid grid-cols-1 gap-4 py-4">
                    <Button
                        variant="outline"
                        className="h-20 flex flex-col items-center justify-center gap-2 hover:border-blue-500 hover:bg-blue-50 dark:hover:bg-blue-950"
                        onClick={() => {
                            onSelect('odt');
                            onOpenChange(false);
                        }}
                    >
                        <div className="flex items-center gap-3 w-full px-2">
                            <div className="bg-blue-100 dark:bg-blue-900 p-2 rounded-lg">
                                <FileText className="h-6 w-6 text-blue-600 dark:text-blue-400" />
                            </div>
                            <div className="text-left">
                                <span className="text-sm font-bold block text-foreground">OpenDocument Text (.odt)</span>
                                <span className="text-[10px] text-muted-foreground">Standard compressed format (recommended)</span>
                            </div>
                        </div>
                    </Button>

                    <Button
                        variant="outline"
                        className="h-20 flex flex-col items-center justify-center gap-2 hover:border-green-500 hover:bg-green-50 dark:hover:bg-green-950"
                        onClick={() => {
                            onSelect('fodt');
                            onOpenChange(false);
                        }}
                    >
                        <div className="flex items-center gap-3 w-full px-2">
                            <div className="bg-green-100 dark:bg-green-900 p-2 rounded-lg">
                                <FileCode className="h-6 w-6 text-green-600 dark:text-green-400" />
                            </div>
                            <div className="text-left">
                                <span className="text-sm font-bold block text-foreground">Flat XML ODT (.fodt)</span>
                                <span className="text-[10px] text-muted-foreground">Single XML file, useful for version control</span>
                            </div>
                        </div>
                    </Button>

                    <Button
                        variant="ghost"
                        className="h-10 text-muted-foreground text-xs"
                        disabled
                    >
                        <FileDown className="h-3 w-3 mr-2" />
                        More formats (OOXML, PDF, EPUB) coming soon
                    </Button>
                </div>
            </DialogContent>
        </Dialog>
    );
}
