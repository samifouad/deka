import { NextResponse } from 'next/server';

export async function GET(req: Request) {
  const res = NextResponse.redirect(new URL('/', req.url));

  res.cookies.delete(`sb-${process.env.SUPABASE_PROJECT_ID}-auth-token`);
  res.cookies.delete(`sb-${process.env.SUPABASE_PROJECT_ID}-auth-token-code-verifier`);

  return res;
}