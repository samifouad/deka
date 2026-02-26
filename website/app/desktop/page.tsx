import { DetailPage } from '@/components/landing/DetailPage'
import { detailPages } from '@/data/landing'

export default function DesktopPage() {
  return <DetailPage {...detailPages.desktop} />
}
