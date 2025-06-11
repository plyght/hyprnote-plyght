export const extractTextFromHtml = (
  html: string | null | undefined,
): string => {
  if (!html) {
    return "";
  }

  const tempDiv = document.createElement("div");
  tempDiv.innerHTML = html;

  const brElements = tempDiv.getElementsByTagName("br");
  for (const br of Array.from(brElements)) {
    br.replaceWith("\n");
  }

  const pElements = tempDiv.getElementsByTagName("p");
  for (const p of Array.from(pElements)) {
    p.innerHTML = p.innerHTML + "\n";
  }

  const textContent = tempDiv.textContent || tempDiv.innerText || "";

  return textContent
    .split("\n")
    .map((line) => line.trim())
    .filter((line) => line.length > 0)
    .join("\n");
};

export const convertHtmlToMarkdown = (
  html: string | null | undefined,
): string => {
  if (!html) {
    return "";
  }

  let markdown = html;

  // Convert headings
  markdown = markdown.replace(/<h([1-6])[^>]*>(.*?)<\/h[1-6]>/gi, (_, level, text) => {
    const hashes = "#".repeat(parseInt(level));
    return `${hashes} ${text.trim()}\n\n`;
  });

  // Convert strong/bold
  markdown = markdown.replace(/<(?:strong|b)[^>]*>(.*?)<\/(?:strong|b)>/gi, "**$1**");

  // Convert emphasis/italic
  markdown = markdown.replace(/<(?:em|i)[^>]*>(.*?)<\/(?:em|i)>/gi, "*$1*");

  // Convert code
  markdown = markdown.replace(/<code[^>]*>(.*?)<\/code>/gi, "`$1`");

  // Convert links
  markdown = markdown.replace(/<a[^>]*href=["']([^"']*)["'][^>]*>(.*?)<\/a>/gi, "[$2]($1)");

  // Convert unordered lists
  markdown = markdown.replace(/<ul[^>]*>(.*?)<\/ul>/gis, (_, content) => {
    return content.replace(/<li[^>]*>(.*?)<\/li>/gis, "- $1\n") + "\n";
  });

  // Convert ordered lists
  markdown = markdown.replace(/<ol[^>]*>(.*?)<\/ol>/gis, (_, content) => {
    let counter = 1;
    return content.replace(/<li[^>]*>(.*?)<\/li>/gis, (_: string, listItem: string) => `${counter++}. ${listItem}\n`)
      + "\n";
  });

  // Convert blockquotes
  markdown = markdown.replace(/<blockquote[^>]*>(.*?)<\/blockquote>/gis, (_, content) => {
    return content.trim().split("\n").map((line: string) => `> ${line.trim()}`).join("\n") + "\n\n";
  });

  // Convert paragraphs
  markdown = markdown.replace(/<p[^>]*>(.*?)<\/p>/gi, "$1\n\n");

  // Convert line breaks
  markdown = markdown.replace(/<br\s*\/?>/gi, "\n");

  // Convert divs (treat as paragraphs)
  markdown = markdown.replace(/<div[^>]*>(.*?)<\/div>/gi, "$1\n\n");

  // Remove remaining HTML tags
  markdown = markdown.replace(/<[^>]*>/g, "");

  // Decode HTML entities
  const tempDiv = document.createElement("div");
  tempDiv.innerHTML = markdown;
  markdown = tempDiv.textContent || tempDiv.innerText || "";

  // Clean up extra whitespace
  markdown = markdown
    .split("\n")
    .map((line) => line.trim())
    .join("\n")
    .replace(/\n{3,}/g, "\n\n")
    .trim();

  return markdown;
};
