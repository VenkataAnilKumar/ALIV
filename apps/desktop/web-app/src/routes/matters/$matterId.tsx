import { createFileRoute, useParams } from '@tanstack/react-router'
import { MatterPage } from '@/aliv/components/MatterPage'

export const Route = createFileRoute('/matters/$matterId')({
  component: MatterRoute,
})

function MatterRoute() {
  const { matterId } = useParams({ from: '/matters/$matterId' })
  // Keyed so switching matters resets tabs and conversation state.
  return <MatterPage key={matterId} matterId={matterId} />
}
