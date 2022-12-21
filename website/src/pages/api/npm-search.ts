import https from 'https';
import { NextApiRequest, NextApiResponse } from 'next';
import httpProxyMiddleware from 'next-http-proxy-middleware';

export const config = {
  api: {
    bodyParser: false,
  },
};

export default function proxy(
  req: NextApiRequest,
  res: NextApiResponse
): Promise<any> {
  return httpProxyMiddleware(req, res, {
    target: 'https://www.npmjs.com',
    changeOrigin: true,
    pathRewrite: [
      {
        patternStr: '^/api/',
        replaceStr: '/search/suggestions',
      },
    ],
    agent: new https.Agent(),
  });
}
