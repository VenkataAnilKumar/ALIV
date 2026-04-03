import { Navigate, createHashRouter } from 'react-router-dom'
import { Shell } from '@/components/layout/Shell'
import { ActionsAuditPage } from '@/pages/ActionsAuditPage'
import { ActionsPreviewPage } from '@/pages/ActionsPreviewPage'
import { BenchmarksPage } from '@/pages/BenchmarksPage'
import { ExtractionPage } from '@/pages/ExtractionPage'
import { IngestionPage } from '@/pages/IngestionPage'
import { MemoryPage } from '@/pages/MemoryPage'
import { QueryPage } from '@/pages/QueryPage'
import { WorkspacePage } from '@/pages/WorkspacePage'

export const router = createHashRouter([
  {
    path: '/',
    element: <Shell />,
    children: [
      { index: true, element: <Navigate to="/workspace" replace /> },
      { path: '/workspace', element: <WorkspacePage /> },
      { path: '/ingestion', element: <IngestionPage /> },
      { path: '/query', element: <QueryPage /> },
      { path: '/extraction', element: <ExtractionPage /> },
      { path: '/actions', element: <ActionsPreviewPage /> },
      { path: '/audit', element: <ActionsAuditPage /> },
      { path: '/memory', element: <MemoryPage /> },
      { path: '/benchmarks', element: <BenchmarksPage /> },
    ],
  },
])