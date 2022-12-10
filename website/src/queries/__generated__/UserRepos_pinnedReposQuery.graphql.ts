/**
 * @generated SignedSource<<91e258f322dc1f1353744c3f1a268fba>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest, Query } from 'relay-runtime';
export type UserRepos_pinnedReposQuery$variables = {};
export type UserRepos_pinnedReposQuery$data = {
  readonly viewer: {
    readonly pinnedItems: {
      readonly nodes: ReadonlyArray<{
        readonly description?: string | null;
        readonly forkCount?: number;
        readonly isPrivate?: boolean;
        readonly nameWithOwner?: string;
        readonly primaryLanguage?: {
          readonly color: string | null;
          readonly name: string;
        } | null;
        readonly stargazers?: {
          readonly totalCount: number;
        };
      } | null> | null;
    };
  };
};
export type UserRepos_pinnedReposQuery = {
  response: UserRepos_pinnedReposQuery$data;
  variables: UserRepos_pinnedReposQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "kind": "Literal",
    "name": "first",
    "value": 6
  },
  {
    "kind": "Literal",
    "name": "types",
    "value": "REPOSITORY"
  }
],
v1 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "nameWithOwner",
  "storageKey": null
},
v2 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "isPrivate",
  "storageKey": null
},
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "description",
  "storageKey": null
},
v4 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v5 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "color",
  "storageKey": null
},
v6 = {
  "alias": null,
  "args": null,
  "concreteType": "StargazerConnection",
  "kind": "LinkedField",
  "name": "stargazers",
  "plural": false,
  "selections": [
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "totalCount",
      "storageKey": null
    }
  ],
  "storageKey": null
},
v7 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "forkCount",
  "storageKey": null
},
v8 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
};
return {
  "fragment": {
    "argumentDefinitions": [],
    "kind": "Fragment",
    "metadata": null,
    "name": "UserRepos_pinnedReposQuery",
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "User",
        "kind": "LinkedField",
        "name": "viewer",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": (v0/*: any*/),
            "concreteType": "PinnableItemConnection",
            "kind": "LinkedField",
            "name": "pinnedItems",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": null,
                "kind": "LinkedField",
                "name": "nodes",
                "plural": true,
                "selections": [
                  {
                    "kind": "InlineFragment",
                    "selections": [
                      (v1/*: any*/),
                      (v2/*: any*/),
                      (v3/*: any*/),
                      {
                        "alias": null,
                        "args": null,
                        "concreteType": "Language",
                        "kind": "LinkedField",
                        "name": "primaryLanguage",
                        "plural": false,
                        "selections": [
                          (v4/*: any*/),
                          (v5/*: any*/)
                        ],
                        "storageKey": null
                      },
                      (v6/*: any*/),
                      (v7/*: any*/)
                    ],
                    "type": "Repository",
                    "abstractKey": null
                  }
                ],
                "storageKey": null
              }
            ],
            "storageKey": "pinnedItems(first:6,types:\"REPOSITORY\")"
          }
        ],
        "storageKey": null
      }
    ],
    "type": "Query",
    "abstractKey": null
  },
  "kind": "Request",
  "operation": {
    "argumentDefinitions": [],
    "kind": "Operation",
    "name": "UserRepos_pinnedReposQuery",
    "selections": [
      {
        "alias": null,
        "args": null,
        "concreteType": "User",
        "kind": "LinkedField",
        "name": "viewer",
        "plural": false,
        "selections": [
          {
            "alias": null,
            "args": (v0/*: any*/),
            "concreteType": "PinnableItemConnection",
            "kind": "LinkedField",
            "name": "pinnedItems",
            "plural": false,
            "selections": [
              {
                "alias": null,
                "args": null,
                "concreteType": null,
                "kind": "LinkedField",
                "name": "nodes",
                "plural": true,
                "selections": [
                  {
                    "alias": null,
                    "args": null,
                    "kind": "ScalarField",
                    "name": "__typename",
                    "storageKey": null
                  },
                  {
                    "kind": "InlineFragment",
                    "selections": [
                      (v1/*: any*/),
                      (v2/*: any*/),
                      (v3/*: any*/),
                      {
                        "alias": null,
                        "args": null,
                        "concreteType": "Language",
                        "kind": "LinkedField",
                        "name": "primaryLanguage",
                        "plural": false,
                        "selections": [
                          (v4/*: any*/),
                          (v5/*: any*/),
                          (v8/*: any*/)
                        ],
                        "storageKey": null
                      },
                      (v6/*: any*/),
                      (v7/*: any*/)
                    ],
                    "type": "Repository",
                    "abstractKey": null
                  },
                  {
                    "kind": "InlineFragment",
                    "selections": [
                      (v8/*: any*/)
                    ],
                    "type": "Node",
                    "abstractKey": "__isNode"
                  }
                ],
                "storageKey": null
              }
            ],
            "storageKey": "pinnedItems(first:6,types:\"REPOSITORY\")"
          },
          (v8/*: any*/)
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "b7cbc42c0cf2519623503143cd1eb2f6",
    "id": null,
    "metadata": {},
    "name": "UserRepos_pinnedReposQuery",
    "operationKind": "query",
    "text": "query UserRepos_pinnedReposQuery {\n  viewer {\n    pinnedItems(first: 6, types: REPOSITORY) {\n      nodes {\n        __typename\n        ... on Repository {\n          nameWithOwner\n          isPrivate\n          description\n          primaryLanguage {\n            name\n            color\n            id\n          }\n          stargazers {\n            totalCount\n          }\n          forkCount\n        }\n        ... on Node {\n          __isNode: __typename\n          id\n        }\n      }\n    }\n    id\n  }\n}\n"
  }
};
})();

(node as any).hash = "7fe50c6469206c5b061aa8a08467dd8b";

export default node;
