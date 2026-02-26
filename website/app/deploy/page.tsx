import { DetailPage } from '@/components/landing/DetailPage'
import { detailPages } from '@/data/landing'

export default function DeployPage() {
  return <DetailPage {...detailPages.deploy} />
}
