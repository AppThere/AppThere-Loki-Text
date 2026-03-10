import { useState, useEffect } from 'react';
import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import type { Metadata } from '@/lib/types/odt';

interface MetadataDialogProps {
    open: boolean;
    onOpenChange: (open: boolean) => void;
    metadata: Metadata;
    onSave: (metadata: Metadata) => void;
}

export function MetadataDialog({
    open,
    onOpenChange,
    metadata,
    onSave,
}: MetadataDialogProps) {
    const [formData, setFormData] = useState<Metadata>(metadata);

    useEffect(() => {
        setFormData(metadata);
    }, [metadata]);

    const handleSave = () => {
        onSave(formData);
        onOpenChange(false);
    };

    const updateField = (field: keyof Metadata, value: string) => {
        setFormData(prev => ({ ...prev, [field]: value || null }));
    };

    return (
        <Dialog open={open} onOpenChange={onOpenChange}>
            <DialogContent className="sm:max-w-[425px]">
                <DialogHeader>
                    <DialogTitle>Document Metadata</DialogTitle>
                    <DialogDescription>
                        Update metadata properties of the current ODT document.
                    </DialogDescription>
                </DialogHeader>

                <div className="grid gap-4 py-4">
                    <div className="grid gap-2">
                        <Label htmlFor="title">Title</Label>
                        <Input
                            id="title"
                            value={formData.title || ''}
                            onChange={(e) => updateField('title', e.target.value)}
                            placeholder="Document Title"
                        />
                    </div>

                    <div className="grid gap-2">
                        <Label htmlFor="creator">Creator / Author</Label>
                        <Input
                            id="creator"
                            value={formData.creator || ''}
                            onChange={(e) => updateField('creator', e.target.value)}
                            placeholder="Author Name"
                        />
                    </div>

                    <div className="grid gap-2">
                        <Label htmlFor="subject">Subject</Label>
                        <Input
                            id="subject"
                            value={formData.subject || ''}
                            onChange={(e) => updateField('subject', e.target.value)}
                            placeholder="Document Subject"
                        />
                    </div>

                    <div className="grid gap-2">
                        <Label htmlFor="description">Description</Label>
                        <Input
                            id="description"
                            value={formData.description || ''}
                            onChange={(e) => updateField('description', e.target.value)}
                            placeholder="Short Description"
                        />
                    </div>
                </div>

                <DialogFooter>
                    <Button variant="outline" onClick={() => onOpenChange(false)}>
                        Cancel
                    </Button>
                    <Button onClick={handleSave}>Save</Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
