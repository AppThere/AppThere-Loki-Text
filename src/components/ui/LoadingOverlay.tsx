import { Loader2 } from "lucide-react";

interface LoadingOverlayProps {
    message?: string;
}

export function LoadingOverlay({ message = "Loading document..." }: LoadingOverlayProps) {
    return (
        <div className="fixed inset-0 z-[100] flex flex-col items-center justify-center bg-background/80 backdrop-blur-sm transition-opacity duration-300">
            <div className="flex flex-col items-center space-y-4 p-6 rounded-2xl bg-card border shadow-xl animate-in fade-in zoom-in duration-300">
                <div className="relative">
                    <div className="absolute inset-0 rounded-full bg-blue-500/20 animate-ping" />
                    <Loader2 className="h-10 w-10 text-blue-500 animate-spin relative z-10" />
                </div>
                <div className="flex flex-col items-center">
                    <p className="text-lg font-bold tracking-tight text-foreground">
                        {message}
                    </p>
                    <p className="text-xs text-muted-foreground animate-pulse">
                        Please wait a moment
                    </p>
                </div>
            </div>
        </div>
    );
}
