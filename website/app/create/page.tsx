import { DetailPage } from '@/components/landing/DetailPage'
import { detailPages } from '@/data/landing'

export default function CreatePage() {
  return <DetailPage {...detailPages.create} />
}
