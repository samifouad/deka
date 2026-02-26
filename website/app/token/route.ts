import { supabase_service } from '@/utils/supabase/server'
import { customAlphabet } from 'nanoid'

export async function POST(req: Request) {

  const sb = supabase_service()

  try {
    // Parse the incoming request body
    const { device_code, metadata } = await req.json();

    if (!device_code) {
      return Response.json({ message: 'missing info' }, { status: 400 });
    }

    const { data, error } = await sb
                                    .from('devices')
                                    .select('token')
                                    .eq('device_id', device_code)

    if (error) {
      throw error; // Handle insert errors
    }

    const token = data[0]

    // For now, we just return a success message
    return Response.json({ token }, { status: 201 });

  } catch (error) {
    console.error('Error processing the device flow request:', error);
    return Response.json({ message: 'Internal Server Error' }, { status: 500 });
  }
}

function normalizeArray(arr: any): any {
  return arr.reduce((acc: any, item: any) => {
    const [key, value] = Object.entries(item)[0];
    acc[key] = value;
    return acc;
  }, {});
}