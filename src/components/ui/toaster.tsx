import { useToast } from '@/lib/hooks/useToast';
import {
    Toast,
    ToastClose,
    ToastDescription,
    ToastProvider,
    ToastTitle,
    ToastViewport,
} from '@/components/ui/toast';

export function Toaster() {
    const { toasts, dismiss } = useToast();

    return (
        <ToastProvider>
            {toasts.map(({ id, title, description, variant, open }) => (
                <Toast
                    key={id}
                    open={open}
                    onOpenChange={(isOpen) => {
                        if (!isOpen) dismiss(id);
                    }}
                    variant={variant}
                >
                    <div className="grid gap-1">
                        {title && <ToastTitle>{title}</ToastTitle>}
                        {description && (
                            <ToastDescription>{description}</ToastDescription>
                        )}
                    </div>
                    <ToastClose />
                </Toast>
            ))}
            <ToastViewport />
        </ToastProvider>
    );
}
