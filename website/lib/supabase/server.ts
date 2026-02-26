// Stub Supabase server module for build compatibility
// TODO: Implement actual Supabase integration

type SupabaseResponse<T = any> = {
  data: T | null
  error: Error | null
}

const createQueryBuilder = () => {
  const builder: any = {
    select: (columns?: string) => builder,
    eq: (column: string, value: any) => builder,
    single: () => Promise.resolve({ data: null, error: new Error('Not implemented') } as SupabaseResponse),
    insert: (values: any) => {
      // insert can be awaited directly OR chained, so we need a "thenable" object
      const insertBuilder = Object.assign(
        Promise.resolve({ data: null, error: new Error('Not implemented') } as SupabaseResponse),
        builder
      )
      return insertBuilder
    },
    update: (values: any) => builder,
    delete: () => builder,
  }
  return builder
}

export const supabase = () => ({
  from: (table: string) => createQueryBuilder(),
  auth: {
    getUser: () => Promise.resolve({ data: { user: null }, error: new Error('Not implemented') }),
    signUp: (credentials: { email: string; password: string }) =>
      Promise.resolve({ data: { user: null }, error: new Error('Not implemented') } as SupabaseResponse<{ user: any }>),
    signIn: (credentials: { email: string; password: string }) =>
      Promise.resolve({ data: { user: null }, error: new Error('Not implemented') } as SupabaseResponse<{ user: any }>),
    stopAutoRefresh: () => {},
    verifyOtp: (params: { type: string; token_hash: string }) =>
      Promise.resolve({ data: null, error: new Error('Not implemented') } as SupabaseResponse),
  }
})

export const supabase_service = () => supabase()