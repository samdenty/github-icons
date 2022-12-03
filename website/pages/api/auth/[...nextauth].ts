import NextAuth, { NextAuthOptions } from 'next-auth';
import GithubProvider from 'next-auth/providers/github';

export const authOptions: NextAuthOptions = {
  // Configure one or more authentication providers
  providers: [
    GithubProvider({
      clientId: process.env.GITHUB_ID!,
      clientSecret: process.env.GITHUB_SECRET!,
    }),
  ],
  callbacks: {
    async jwt({ token, account }) {
      token.accessToken = account!.access_token!;
      return token;
    },
    async session({ session, token, user }) {
      return {
        ...session,
        user,
        accessToken: token.accessToken,
      };
    },
  },
};

export default NextAuth(authOptions);
