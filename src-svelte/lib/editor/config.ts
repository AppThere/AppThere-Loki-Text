import StarterKit from '@tiptap/starter-kit';
import Underline from '@tiptap/extension-underline';
import Link from '@tiptap/extension-link';
import Superscript from '@tiptap/extension-superscript';
import Subscript from '@tiptap/extension-subscript';
import Image from '@tiptap/extension-image';
import { Table } from '@tiptap/extension-table';
import TableRow from '@tiptap/extension-table-row';
import TableCell from '@tiptap/extension-table-cell';
import TableHeader from '@tiptap/extension-table-header';
import BulletList from '@tiptap/extension-bullet-list';
import OrderedList from '@tiptap/extension-ordered-list';
import ListItem from '@tiptap/extension-list-item';
import Blockquote from '@tiptap/extension-blockquote';
import TextAlign from '@tiptap/extension-text-align';
import { NamedSpanStyle, NamedBlockStyle } from '../extensions/NamedStyles';
import { NextParagraphStyle } from '../extensions/NextParagraphStyle';
import { PageBreak } from '../extensions/PageBreak';

export function getEditorExtensions() {
	return [
		StarterKit.configure({
			bulletList: false,
			orderedList: false,
			listItem: false,
			blockquote: false
		}),
		// Underline, // Commented out in original
		Superscript,
		Subscript,
		// Link.configure({...}), // Commented out in original
		Image,
		Table.configure({
			resizable: true
		}),
		TableRow,
		TableHeader,
		TableCell,
		BulletList,
		OrderedList,
		ListItem,
		Blockquote,
		TextAlign.configure({
			types: ['heading', 'paragraph']
		}),
		NamedBlockStyle,
		NextParagraphStyle,
		PageBreak
	];
}
