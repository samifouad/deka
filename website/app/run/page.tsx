import { DetailPage } from '@/components/landing/DetailPage'
import { detailPages } from '@/data/landing'

export default function RunPage() {
  return <DetailPage {...detailPages.run} />
}
