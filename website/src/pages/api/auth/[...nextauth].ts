import NextAuth, { NextAuthOptions } from 'next-auth';
import { JWTOptions } from 'next-auth/jwt';
import GithubProvider from 'next-auth/providers/github';

const plainTextJWT: Partial<JWTOptions> = {
  encode({ token }) {
    return JSON.stringify(token);
  },
  decode({ token }) {
    return token && JSON.parse(token);
  },
};

export const authOptions: NextAuthOptions = {
  // Configure one or more authentication providers
  providers: [
    GithubProvider({
      clientId: process.env.GITHUB_ID!,
      clientSecret: process.env.GITHUB_SECRET!,
      authorization: {
        params: {
          scope: 'read:org',
        },
      },
      profile(profile) {
        return {
          id: profile.login,
          email: profile.email,
          name: profile.name,
          image: profile.avatar_url,
        };
      },
    }),
  ],
  jwt: process.env.NODE_ENV === 'development' ? plainTextJWT : undefined,
  callbacks: {
    async jwt({ token, account }) {
      if (account?.access_token) {
        token.accessToken = account.access_token;
      }
      return token;
    },
    async session({ session, token }) {
      session.user.id = token.sub!;

      return {
        ...session,
        accessToken: token.accessToken,
      };
    },
  },
};

export default NextAuth(authOptions);
