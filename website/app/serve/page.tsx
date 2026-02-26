import { DetailPage } from '@/components/landing/DetailPage'
import { detailPages } from '@/data/landing'

export default function ServePage() {
  return <DetailPage {...detailPages.serve} />
}
