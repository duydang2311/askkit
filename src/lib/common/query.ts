import { useQueryClient, type InvalidateOptions, type InvalidateQueryFilters, type QueryKey } from "@tanstack/svelte-query";

export const invalidateQueries = <TTaggedQueryKey extends QueryKey = QueryKey>(filters?: InvalidateQueryFilters<TTaggedQueryKey>, options?: InvalidateOptions) => {
  return useQueryClient().invalidateQueries(filters, options);
}