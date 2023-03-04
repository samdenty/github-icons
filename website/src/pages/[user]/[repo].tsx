import Modal from 'react-modal';
import { useRouter } from 'next/router';
import { Repo } from '../../components/Repo/Repo';

export default function RepoPage() {
  const router = useRouter();
  const { user, repo } = router.query;

  if (!repo) {
    return null;
  }

  return <Repo slug={`${user}/${repo}`} />;
}
