import { useState } from "react";
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { readFile } from '@tauri-apps/plugin-fs';

interface ImageDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    onSave: (src: string, alt: string) => void;
}

export function ImageDialog({ open, onOpenChange, onSave }: ImageDialogProps) {
    const [url, setUrl] = useState("");
    const [alt, setAlt] = useState("");
    const [fileName, setFileName] = useState("");

    const handleFileSelect = async () => {
        try {
            const selected = await openDialog({
                multiple: false,
                filters: [{
                    name: 'Image',
                    extensions: ['png', 'jpeg', 'jpg', 'gif', 'webp']
                }]
            });

            if (selected && typeof selected === 'string') {
                setFileName(selected.split('/').pop() || selected.split('\\').pop() || 'Image');
                const contents = await readFile(selected);

                let binary = '';
                const len = contents.byteLength;
                // Reading chunked to avoid stack overflow on huge files
                const chunkSize = 8192;
                for (let i = 0; i < len; i += chunkSize) {
                    const chunk = contents.subarray(i, i + chunkSize);
                    binary += String.fromCharCode.apply(null, chunk as unknown as number[]);
                }
                const base64 = btoa(binary);

                const ext = selected.split('.').pop()?.toLowerCase();
                const mimeType = ext === 'jpg' ? 'jpeg' : ext;
                setUrl(`data:image/${mimeType};base64,${base64}`);
            }
        } catch (e) {
            console.error("Failed to read image:", e);
        }
    };

    const handleSave = () => {
        if (url) {
            onSave(url, alt || fileName || "Image");
            onOpenChange(false);
            setUrl("");
            setAlt("");
            setFileName("");
        }
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle>Insert Image</DialogTitle>
                </DialogHeader>

                <Tabs defaultValue="upload" className="w-full">
                    <TabsList className="grid w-full grid-cols-2">
                        <TabsTrigger value="upload">Upload</TabsTrigger>
                        <TabsTrigger value="url">URL</TabsTrigger>
                    </TabsList>
                    <TabsContent value="upload" className="space-y-4 py-4">
                        <div className="flex flex-col items-center justify-center border-2 border-dashed border-gray-300 dark:border-gray-700 rounded-md p-6">
                            {url && url.startsWith('data:') ? (
                                <div className="text-sm text-green-600 dark:text-green-400 mb-4">
                                    Image Selected: {fileName || "Local Image"}
                                </div>
                            ) : (
                                <div className="text-sm text-gray-500 mb-4">
                                    No image selected
                                </div>
                            )}
                            <Button onClick={handleFileSelect} variant="secondary">
                                Browse Files
                            </Button>
                        </div>
                    </TabsContent>
                    <TabsContent value="url" className="space-y-4 py-4">
                        <div className="grid grid-cols-4 items-center gap-4">
                            <Label htmlFor="imgUrl" className="text-right">
                                URL
                            </Label>
                            <Input
                                id="imgUrl"
                                value={url}
                                onChange={(e) => setUrl(e.target.value)}
                                className="col-span-3"
                                placeholder="https://example.com/image.png"
                            />
                        </div>
                    </TabsContent>
                </Tabs>

                <div className="grid grid-cols-4 items-center gap-4 py-2">
                    <Label htmlFor="altText" className="text-right">
                        Alt Text
                    </Label>
                    <Input
                        id="altText"
                        value={alt}
                        onChange={(e) => setAlt(e.target.value)}
                        className="col-span-3"
                        placeholder="Image description"
                    />
                </div>

                <DialogFooter>
                    <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
                    <Button type="submit" onClick={handleSave} disabled={!url}>Insert</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
