import { createFileRoute } from '@tanstack/react-router'
import { MattersPage } from '@/aliv/components/MattersPage'

export const Route = createFileRoute('/matters/')({
  component: MattersPage,
})
