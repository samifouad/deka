import { DetailPage } from '@/components/landing/DetailPage'
import { detailPages } from '@/data/landing'

export default function IntrospectPage() {
  return <DetailPage {...detailPages.introspect} />
}
