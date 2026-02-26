import { DetailPage } from '@/components/landing/DetailPage'
import { detailPages } from '@/data/landing'

export default function CompilePage() {
  return <DetailPage {...detailPages.compile} />
}
