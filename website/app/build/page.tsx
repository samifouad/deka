import { DetailPage } from '@/components/landing/DetailPage'
import { detailPages } from '@/data/landing'

export default function BuildPage() {
  return <DetailPage {...detailPages.build} />
}
