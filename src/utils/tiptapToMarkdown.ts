// Converts a Tiptap JSON document to Markdown text.

type TiptapNode = {
  type: string
  attrs?: Record<string, unknown>
  content?: TiptapNode[]
  marks?: { type: string; attrs?: Record<string, unknown> }[]
  text?: string
}

const applyMarks = (text: string, marks: TiptapNode['marks']): string => {
  if (!marks?.length) return text
  let result = text
  for (const mark of marks) {
    switch (mark.type) {
      case 'bold': result = `**${result}**`; break
      case 'italic': result = `*${result}*`; break
      case 'code': result = `\`${result}\``; break
      case 'strike': result = `~~${result}~~`; break
      case 'link': result = `[${result}](${mark.attrs?.href ?? ''})`; break
    }
  }
  return result
}

const nodeToMd = (node: TiptapNode, listDepth = 0, listType = ''): string => {
  switch (node.type) {
    case 'doc':
      return (node.content ?? []).map(n => nodeToMd(n)).join('\n')

    case 'paragraph': {
      const text = (node.content ?? []).map(n => nodeToMd(n)).join('')
      return text || ''
    }

    case 'heading': {
      const level = (node.attrs?.level as number) ?? 1
      const text = (node.content ?? []).map(n => nodeToMd(n)).join('')
      return `${'#'.repeat(level)} ${text}`
    }

    case 'bulletList':
      return (node.content ?? []).map(n => nodeToMd(n, listDepth, 'bullet')).join('\n')

    case 'orderedList':
      return (node.content ?? []).map((n, i) => nodeToMd(n, listDepth, `${i + 1}.`)).join('\n')

    case 'taskList':
      return (node.content ?? []).map(n => nodeToMd(n, listDepth, 'task')).join('\n')

    case 'listItem': {
      const indent = '  '.repeat(listDepth)
      const bullet = listType === 'bullet' ? '-' : listType
      const inner = (node.content ?? []).map(n => {
        if (n.type === 'bulletList' || n.type === 'orderedList' || n.type === 'taskList')
          return '\n' + nodeToMd(n, listDepth + 1, n.type === 'orderedList' ? '1.' : n.type === 'taskList' ? 'task' : 'bullet')
        return nodeToMd(n)
      }).join('')
      return `${indent}${bullet} ${inner}`
    }

    case 'taskItem': {
      const indent = '  '.repeat(listDepth)
      const checked = node.attrs?.checked ? 'x' : ' '
      const inner = (node.content ?? []).map(n => nodeToMd(n)).join('')
      return `${indent}- [${checked}] ${inner}`
    }

    case 'blockquote': {
      const inner = (node.content ?? []).map(n => nodeToMd(n)).join('\n')
      return inner.split('\n').map(l => `> ${l}`).join('\n')
    }

    case 'codeBlock': {
      const lang = (node.attrs?.language as string) ?? ''
      const code = (node.content ?? []).map(n => n.text ?? '').join('')
      return `\`\`\`${lang}\n${code}\n\`\`\``
    }

    case 'horizontalRule':
      return '---'

    case 'hardBreak':
      return '  \n'

    case 'text':
      return applyMarks(node.text ?? '', node.marks)

    case 'image': {
      const src = (node.attrs?.src as string) ?? ''
      const alt = (node.attrs?.alt as string) ?? ''
      return `![${alt}](${src})`
    }

    default:
      return (node.content ?? []).map(n => nodeToMd(n)).join('')
  }
}

export const tiptapToMarkdown = (doc: unknown): string => {
  const lines = nodeToMd(doc as TiptapNode).split('\n')
  // Collapse more than 2 consecutive blank lines into 2
  const result: string[] = []
  let blanks = 0
  for (const line of lines) {
    if (line.trim() === '') {
      blanks++
      if (blanks <= 2) result.push(line)
    } else {
      blanks = 0
      result.push(line)
    }
  }
  return result.join('\n').trim() + '\n'
}
