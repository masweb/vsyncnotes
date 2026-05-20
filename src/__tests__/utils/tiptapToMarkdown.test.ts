import { describe, it, expect } from 'vitest'
import { tiptapToMarkdown } from '@/utils/tiptapToMarkdown'

describe('tiptapToMarkdown', () => {
  it('converts a simple paragraph', () => {
    const doc = {
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [{ type: 'text', text: 'Hello world' }]
        }
      ]
    }
    expect(tiptapToMarkdown(doc)).toBe('Hello world\n')
  })

  it('converts a heading', () => {
    const doc = {
      type: 'doc',
      content: [
        {
          type: 'heading',
          attrs: { level: 2 },
          content: [{ type: 'text', text: 'Title' }]
        }
      ]
    }
    expect(tiptapToMarkdown(doc)).toBe('## Title\n')
  })

  it('converts bold text', () => {
    const doc = {
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            { type: 'text', text: 'bold', marks: [{ type: 'bold' }] }
          ]
        }
      ]
    }
    expect(tiptapToMarkdown(doc)).toBe('**bold**\n')
  })

  it('converts a bullet list', () => {
    const doc = {
      type: 'doc',
      content: [
        {
          type: 'bulletList',
          content: [
            {
              type: 'listItem',
              content: [
                { type: 'paragraph', content: [{ type: 'text', text: 'one' }] }
              ]
            },
            {
              type: 'listItem',
              content: [
                { type: 'paragraph', content: [{ type: 'text', text: 'two' }] }
              ]
            }
          ]
        }
      ]
    }
    expect(tiptapToMarkdown(doc)).toBe('- one\n- two\n')
  })

  it('converts a code block with language', () => {
    const doc = {
      type: 'doc',
      content: [
        {
          type: 'codeBlock',
          attrs: { language: 'ts' },
          content: [{ type: 'text', text: 'const x = 1' }]
        }
      ]
    }
    expect(tiptapToMarkdown(doc)).toBe('```ts\nconst x = 1\n```\n')
  })

  it('converts a blockquote', () => {
    const doc = {
      type: 'doc',
      content: [
        {
          type: 'blockquote',
          content: [
            { type: 'paragraph', content: [{ type: 'text', text: 'quote me' }] }
          ]
        }
      ]
    }
    expect(tiptapToMarkdown(doc)).toBe('> quote me\n')
  })

  it('converts a horizontal rule', () => {
    const doc = {
      type: 'doc',
      content: [
        { type: 'horizontalRule' }
      ]
    }
    expect(tiptapToMarkdown(doc)).toBe('---\n')
  })

  it('converts an image', () => {
    const doc = {
      type: 'doc',
      content: [
        {
          type: 'image',
          attrs: { src: 'img.png', alt: 'alt text' }
        }
      ]
    }
    expect(tiptapToMarkdown(doc)).toBe('![alt text](img.png)\n')
  })

  it('converts a link', () => {
    const doc = {
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            {
              type: 'text',
              text: 'click',
              marks: [{ type: 'link', attrs: { href: 'https://example.com' } }]
            }
          ]
        }
      ]
    }
    expect(tiptapToMarkdown(doc)).toBe('[click](https://example.com)\n')
  })

  it('converts a task list', () => {
    const doc = {
      type: 'doc',
      content: [
        {
          type: 'taskList',
          content: [
            {
              type: 'taskItem',
              attrs: { checked: true },
              content: [
                { type: 'paragraph', content: [{ type: 'text', text: 'done' }] }
              ]
            },
            {
              type: 'taskItem',
              attrs: { checked: false },
              content: [
                { type: 'paragraph', content: [{ type: 'text', text: 'todo' }] }
              ]
            }
          ]
        }
      ]
    }
    expect(tiptapToMarkdown(doc)).toBe('- [x] done\n- [ ] todo\n')
  })

  it('handles empty doc', () => {
    const doc = { type: 'doc', content: [] }
    expect(tiptapToMarkdown(doc)).toBe('\n')
  })

  it('converts italic and strike marks', () => {
    const doc = {
      type: 'doc',
      content: [
        {
          type: 'paragraph',
          content: [
            { type: 'text', text: 'ital', marks: [{ type: 'italic' }] },
            { type: 'text', text: ' ' },
            { type: 'text', text: 'gone', marks: [{ type: 'strike' }] }
          ]
        }
      ]
    }
    expect(tiptapToMarkdown(doc)).toBe('*ital* ~~gone~~\n')
  })

  it('converts an ordered list', () => {
    const doc = {
      type: 'doc',
      content: [
        {
          type: 'orderedList',
          content: [
            {
              type: 'listItem',
              content: [
                { type: 'paragraph', content: [{ type: 'text', text: 'first' }] }
              ]
            },
            {
              type: 'listItem',
              content: [
                { type: 'paragraph', content: [{ type: 'text', text: 'second' }] }
              ]
            }
          ]
        }
      ]
    }
    expect(tiptapToMarkdown(doc)).toBe('1. first\n2. second\n')
  })
})
