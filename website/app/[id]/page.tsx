import { getProfile } from '@/lib/getProfile'
import { getProjects } from '@/lib/getProjects'
import Link from 'next/link'

export default async function Page(props: { params: Promise<{ id: string }> }) {
  const params = await props.params;

  const { data: userProfile, error: errorUser } = await getProfile(params.id)

  const { data: userProjects, error: errorProjects } = await getProjects()

  if(errorUser !== null) {
          console.log('errorUser: '+ JSON.stringify(errorUser))
          return <p>access denied</p>
  }

  if(errorProjects !== null) {
          console.log('errorProjects: '+ JSON.stringify(errorProjects))
          return <p>access denied</p>
  }

  console.log('userProfile: '+ JSON.stringify(userProfile))

  return (
   <div className="flex items-start p-10">
     {userProjects && userProjects.length > 0 ? (
       userProjects.map((project) => (
         <Link key={project.project_id} href={`/${params.id}/${project.slug}`}>
           <div className="w-[220px] h-[220px] mr-8 bg-slate-200 p-5 border-slate-500">
             <p>Name: {project.name}</p>
           </div>
         </Link>
       ))
     ) : (
       <p>No projects found.</p>
     )}
   </div>
 )
}