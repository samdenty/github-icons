import { useRouter } from 'next/router';
import { AllIcons } from '../../components/AllIcons/AllIcons';

export default function RepoPage() {
  const router = useRouter();
  const segments = router.query.pkg as string[];

  if (!segments) {
    return null;
  }

  const org: string | undefined = segments[1] && segments[0];
  const pkg = segments[1] || segments[0];

  if (!pkg) {
    return null;
  }

  return <AllIcons type="npm" slug={[org, pkg].filter(Boolean).join('/')} />;
}
