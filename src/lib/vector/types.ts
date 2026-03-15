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

/**
 * A colour value. The `type` field discriminates the variant.
 * Field names and variant names must match the Rust Colour enum exactly —
 * see src-tauri/formats/common-core/src/colour_management/colour.rs.
 */
export type Colour =
    | { type: 'Rgb'; r: number; g: number; b: number; a: number }
    | { type: 'Cmyk'; c: number; m: number; y: number; k: number; alpha: number }
    | { type: 'Lab'; l: number; a: number; b: number; alpha: number }
    | {
          type: 'Spot';
          name: string;
          tint: number;
          lab_ref: [number, number, number];
          cmyk_fallback: Colour;
      }
    | { type: 'Linked'; id: string };

export type BuiltInProfile =
    | 'SrgbIec61966'
    | 'IsoCoatedV2'
    | 'SwopV2'
    | 'GraCol2006';

export type IccProfileRef =
    | { type: 'BuiltIn'; profile: BuiltInProfile }
    | { type: 'FilePath'; path: string };

export type ColourSpace =
    | { type: 'Srgb' }
    | { type: 'DisplayP3' }
    | { type: 'AdobeRgb' }
    | { type: 'Cmyk'; profile: IccProfileRef }
    | { type: 'Custom'; profile: IccProfileRef };

export type RenderingIntent =
    | 'Perceptual'
    | 'RelativeColorimetric'
    | 'Saturation'
    | 'AbsoluteColorimetric';

export interface DocumentColourSettings {
    working_space: ColourSpace;
    rendering_intent: RenderingIntent;
    blackpoint_compensation: boolean;
}

export interface SwatchId {
    id: string;
}

export interface ColourSwatch {
    id: SwatchId;
    name: string;
    colour: Colour;
    is_spot: boolean;
}

export interface SwatchLibrary {
    swatches: ColourSwatch[];
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
    colour_settings: DocumentColourSettings;
}

export function identityTransform(): Transform {
    return { a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 };
}

export function defaultStyle(): ObjectStyle {
    return {
        fill: { type: 'Solid', colour: { type: 'Rgb', r: 0, g: 0, b: 0, a: 1 } },
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
