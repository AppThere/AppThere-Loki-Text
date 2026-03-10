import { useState, useEffect } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

interface LinkDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    initialUrl: string;
    onSave: (url: string | null) => void;
}

export function LinkDialog({ open, onOpenChange, initialUrl, onSave }: LinkDialogProps) {
    const [url, setUrl] = useState(initialUrl);

    useEffect(() => {
        if (open) {
            setUrl(initialUrl);
        }
    }, [open, initialUrl]);

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle>{initialUrl ? 'Edit Link' : 'Insert Link'}</DialogTitle>
                </DialogHeader>
                <div className="grid gap-4 py-4">
                    <div className="grid grid-cols-4 items-center gap-4">
                        <Label htmlFor="url" className="text-right">
                            URL
                        </Label>
                        <Input
                            id="url"
                            value={url}
                            onChange={(e) => setUrl(e.target.value)}
                            className="col-span-3"
                            placeholder="https://example.com"
                            autoFocus
                        />
                    </div>
                </div>
                <DialogFooter>
                    {initialUrl && (
                        <Button type="button" variant="destructive" className="mr-auto" onClick={() => { onSave(null); onOpenChange(false); }}>
                            Remove Link
                        </Button>
                    )}
                    <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
                    <Button type="submit" onClick={() => { onSave(url); onOpenChange(false); }}>Save</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
