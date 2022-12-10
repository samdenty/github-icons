import NextAuth, { NextAuthOptions } from 'next-auth';
import GithubProvider from 'next-auth/providers/github';

export const authOptions: NextAuthOptions = {
  // Configure one or more authentication providers
  providers: [
    GithubProvider({
      clientId: process.env.GITHUB_ID!,
      clientSecret: process.env.GITHUB_SECRET!,
      authorization: {
        params: {
          scope: 'repo',
        },
      },
    }),
  ],
  jwt:
    process.env.NODE_ENV === 'development'
      ? {
          encode({ token }) {
            return JSON.stringify(token);
          },
          decode({ token }) {
            return JSON.parse(token);
          },
        }
      : undefined,
  callbacks: {
    async jwt({ token, account }) {
      if (account?.access_token) {
        token.accessToken = account.access_token;
      }
      return token;
    },
    async session({ session, token }) {
      return {
        ...session,
        accessToken: token.accessToken,
      };
    },
  },
};

export default NextAuth(authOptions);
