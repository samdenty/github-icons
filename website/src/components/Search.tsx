import styled from '@emotion/styled';
import { signIn, useSession } from 'next-auth/react';
import { BsGithub } from 'react-icons/bs';

const StyledSearch = styled.div`
  position: relative;
  background: #ffffff33;
  border-radius: 7px;
  backdrop-filter: blur(10px);
  width: 750px;
  margin-bottom: 15px;

  @media (max-width: 1320px) {
    width: 80%;
  }
`;

const Input = styled.input`
  background: transparent;
  height: 100%;
  width: 100%;
  padding: 15px 10px;
  outline: none;
  border: none;
  text-align: center;

  &:disabled {
    cursor: pointer;
  }
`;

const SignIn = styled.button`
  position: absolute;
  width: 100%;
  height: 100%;
  top: 0;
  left: 0;
  background: #23282c;
  color: #fff;
  opacity: 0;
  cursor: pointer;
  border: none;
  padding: 10px 8px;
  justify-content: center;
  display: flex;
  align-items: center;
  transition: opacity 0.1s ease;
  border-radius: 7px;

  > * {
    margin-right: 10px;
    height: 24px;
    width: 24px;
  }

  ${StyledSearch}:hover &,
  ${StyledSearch}:focus-within & {
    opacity: 1;
  }
`;

export interface SearchProps {
  onQuery(query: string): void;
  query: string;
  placeholder?: string;
}

export default function Search({ onQuery, query, placeholder }: SearchProps) {
  const { data: session } = useSession();

  return (
    <StyledSearch>
      <Input
        autoFocus
        disabled={!session}
        placeholder={placeholder}
        value={query}
        onChange={(e) => {
          onQuery(e.target.value);
        }}
        autoComplete="off"
        autoCorrect="off"
        spellCheck={false}
      />
      {!session && (
        <SignIn
          onClick={() => {
            signIn('github');
          }}
        >
          <BsGithub />
          Sign In with GitHub to search (to prevent rate-limiting)
        </SignIn>
      )}
    </StyledSearch>
  );
}
