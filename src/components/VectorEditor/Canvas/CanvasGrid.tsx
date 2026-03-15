import { useMemo } from 'react';
import { Line } from 'react-konva';

interface CanvasGridProps {
    width: number;   // stage width in screen px
    height: number;  // stage height in screen px
    zoom: number;
    panX: number;
    panY: number;
    spacingPx: number; // base grid spacing in document coords
}

export function CanvasGrid({ width, height, zoom, panX, panY, spacingPx }: CanvasGridProps) {
    const lines = useMemo(() => {
        // Adapt spacing based on zoom level
        let spacing = spacingPx;
        if (zoom < 0.25) spacing *= 4;
        else if (zoom < 0.5) spacing *= 2;
        else if (zoom > 4) spacing *= 0.5;

        const screenSpacing = spacing * zoom;
        if (screenSpacing < 4) return null; // Too dense to draw

        const result: React.ReactElement[] = [];
        const color = 'rgba(100,100,100,0.2)';

        // Offset of the origin in screen coords
        const originX = panX;
        const originY = panY;

        // First grid line index
        const startXi = Math.floor(-originX / screenSpacing) - 1;
        const endXi = Math.ceil((width - originX) / screenSpacing) + 1;
        const startYi = Math.floor(-originY / screenSpacing) - 1;
        const endYi = Math.ceil((height - originY) / screenSpacing) + 1;

        for (let i = startXi; i <= endXi; i++) {
            const x = originX + i * screenSpacing;
            result.push(
                <Line
                    key={`vg-${i}`}
                    points={[x, 0, x, height]}
                    stroke={color}
                    strokeWidth={1}
                    listening={false}
                />,
            );
        }

        for (let j = startYi; j <= endYi; j++) {
            const y = originY + j * screenSpacing;
            result.push(
                <Line
                    key={`hg-${j}`}
                    points={[0, y, width, y]}
                    stroke={color}
                    strokeWidth={1}
                    listening={false}
                />,
            );
        }

        return result;
    }, [width, height, zoom, panX, panY, spacingPx]);

    return <>{lines}</>;
}
