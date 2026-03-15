// TypeScript mirror of the Rust VectorDocument types.
// Field names match the Rust struct field names exactly (serde snake_case defaults).

export type LengthUnit = 'Px' | 'Mm' | 'Cm' | 'In' | 'Pt' | 'Pc';

export interface ViewBox {
    x: number;
    y: number;
    width: number;
    height: number;
}

export interface Canvas {
    width: number;    // px
    height: number;   // px
    display_unit: LengthUnit;
    dpi: number;
    viewbox: ViewBox | null;
}

export interface Colour {
    r: number;
    g: number;
    b: number;
    a: number;
}

export interface Transform {
    a: number;
    b: number;
    c: number;
    d: number;
    e: number;
    f: number;
}

export type Paint =
    | { type: 'None' }
    | { type: 'Solid'; colour: Colour };

export type LineCap = 'Butt' | 'Round' | 'Square';
export type LineJoin = 'Miter' | 'Round' | 'Bevel';

export interface StrokeStyle {
    paint: Paint;
    width: number;
    line_cap: LineCap;
    line_join: LineJoin;
    miter_limit: number;
    dash_array: number[];
    dash_offset: number;
}

export interface ObjectStyle {
    fill: Paint;
    stroke: StrokeStyle;
    opacity: number;
    fill_opacity: number;
    stroke_opacity: number;
}

export interface CommonProps {
    id: string;
    label: string | null;
    style: ObjectStyle;
    transform: Transform;
    visible: boolean;
    locked: boolean;
}

export interface RectObject extends CommonProps {
    type: 'Rect';
    x: number;
    y: number;
    width: number;
    height: number;
    rx: number;
    ry: number;
}

export interface EllipseObject extends CommonProps {
    type: 'Ellipse';
    cx: number;
    cy: number;
    rx: number;
    ry: number;
}

export interface LineObject extends CommonProps {
    type: 'Line';
    x1: number;
    y1: number;
    x2: number;
    y2: number;
}

export interface PathObject extends CommonProps {
    type: 'Path';
    d: string;
}

export interface GroupObject extends CommonProps {
    type: 'Group';
    children: VectorObject[];
}

export type VectorObject =
    | RectObject
    | EllipseObject
    | LineObject
    | PathObject
    | GroupObject;

export interface Layer {
    id: string;
    name: string;
    visible: boolean;
    locked: boolean;
    objects: VectorObject[];
}

export interface VectorDocument {
    canvas: Canvas;
    layers: Layer[];
    metadata: {
        title: string | null;
        creator: string | null;
        description: string | null;
        [key: string]: unknown;
    };
}

export function identityTransform(): Transform {
    return { a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 };
}

export function defaultStyle(): ObjectStyle {
    return {
        fill: { type: 'Solid', colour: { r: 0, g: 0, b: 0, a: 255 } },
        stroke: {
            paint: { type: 'None' },
            width: 1,
            line_cap: 'Butt',
            line_join: 'Miter',
            miter_limit: 4,
            dash_array: [],
            dash_offset: 0,
        },
        opacity: 1,
        fill_opacity: 1,
        stroke_opacity: 1,
    };
}
