export const pageFromEnvelope = (body) => {
  const page = body && body.page ? body.page : {};
  return {
    hasMore: Boolean(page.has_more),
    nextCursor: page.next_cursor || null,
  };
};

export const itemsFromEnvelope = (body) =>
  Array.isArray(body && body.items) ? body.items : [];
