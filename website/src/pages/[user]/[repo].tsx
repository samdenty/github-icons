import { useRouter } from 'next/router';
import { AllIcons } from '../../components/AllIcons/AllIcons';

export default function RepoPage() {
  const router = useRouter();
  const { user, repo } = router.query;

  if (!repo) {
    return null;
  }

  return <AllIcons type="github" slug={`${user}/${repo}`} />;
}
