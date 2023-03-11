/**
 * @generated SignedSource<<3949870f48764397a72020c9c408adbf>>
 * @lightSyntaxTransform
 * @nogrep
 */

/* tslint:disable */
/* eslint-disable */
// @ts-nocheck

import { ConcreteRequest, Query } from 'relay-runtime';
export type UserRepos_pinnedReposQuery$variables = {
  user: string;
};
export type UserRepos_pinnedReposQuery$data = {
  readonly repositoryOwner: {
    readonly pinnedItems?: {
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
  } | null;
};
export type UserRepos_pinnedReposQuery = {
  response: UserRepos_pinnedReposQuery$data;
  variables: UserRepos_pinnedReposQuery$variables;
};

const node: ConcreteRequest = (function(){
var v0 = [
  {
    "defaultValue": null,
    "kind": "LocalArgument",
    "name": "user"
  }
],
v1 = [
  {
    "kind": "Variable",
    "name": "login",
    "variableName": "user"
  }
],
v2 = [
  {
    "kind": "Literal",
    "name": "first",
    "value": 6
  },
  {
    "kind": "Literal",
    "name": "types",
    "value": [
      "REPOSITORY"
    ]
  }
],
v3 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "nameWithOwner",
  "storageKey": null
},
v4 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "isPrivate",
  "storageKey": null
},
v5 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "description",
  "storageKey": null
},
v6 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "name",
  "storageKey": null
},
v7 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "color",
  "storageKey": null
},
v8 = {
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
v9 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "forkCount",
  "storageKey": null
},
v10 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "__typename",
  "storageKey": null
},
v11 = {
  "alias": null,
  "args": null,
  "kind": "ScalarField",
  "name": "id",
  "storageKey": null
};
return {
  "fragment": {
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Fragment",
    "metadata": null,
    "name": "UserRepos_pinnedReposQuery",
    "selections": [
      {
        "alias": null,
        "args": (v1/*: any*/),
        "concreteType": null,
        "kind": "LinkedField",
        "name": "repositoryOwner",
        "plural": false,
        "selections": [
          {
            "kind": "InlineFragment",
            "selections": [
              {
                "alias": null,
                "args": (v2/*: any*/),
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
                          (v3/*: any*/),
                          (v4/*: any*/),
                          (v5/*: any*/),
                          {
                            "alias": null,
                            "args": null,
                            "concreteType": "Language",
                            "kind": "LinkedField",
                            "name": "primaryLanguage",
                            "plural": false,
                            "selections": [
                              (v6/*: any*/),
                              (v7/*: any*/)
                            ],
                            "storageKey": null
                          },
                          (v8/*: any*/),
                          (v9/*: any*/)
                        ],
                        "type": "Repository",
                        "abstractKey": null
                      }
                    ],
                    "storageKey": null
                  }
                ],
                "storageKey": "pinnedItems(first:6,types:[\"REPOSITORY\"])"
              }
            ],
            "type": "ProfileOwner",
            "abstractKey": "__isProfileOwner"
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
    "argumentDefinitions": (v0/*: any*/),
    "kind": "Operation",
    "name": "UserRepos_pinnedReposQuery",
    "selections": [
      {
        "alias": null,
        "args": (v1/*: any*/),
        "concreteType": null,
        "kind": "LinkedField",
        "name": "repositoryOwner",
        "plural": false,
        "selections": [
          (v10/*: any*/),
          {
            "kind": "InlineFragment",
            "selections": [
              {
                "alias": null,
                "args": (v2/*: any*/),
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
                      (v10/*: any*/),
                      {
                        "kind": "InlineFragment",
                        "selections": [
                          (v3/*: any*/),
                          (v4/*: any*/),
                          (v5/*: any*/),
                          {
                            "alias": null,
                            "args": null,
                            "concreteType": "Language",
                            "kind": "LinkedField",
                            "name": "primaryLanguage",
                            "plural": false,
                            "selections": [
                              (v6/*: any*/),
                              (v7/*: any*/),
                              (v11/*: any*/)
                            ],
                            "storageKey": null
                          },
                          (v8/*: any*/),
                          (v9/*: any*/)
                        ],
                        "type": "Repository",
                        "abstractKey": null
                      },
                      {
                        "kind": "InlineFragment",
                        "selections": [
                          (v11/*: any*/)
                        ],
                        "type": "Node",
                        "abstractKey": "__isNode"
                      }
                    ],
                    "storageKey": null
                  }
                ],
                "storageKey": "pinnedItems(first:6,types:[\"REPOSITORY\"])"
              }
            ],
            "type": "ProfileOwner",
            "abstractKey": "__isProfileOwner"
          },
          (v11/*: any*/)
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "3fa4edfed8b8818c53a65ad0aaaec175",
    "id": null,
    "metadata": {},
    "name": "UserRepos_pinnedReposQuery",
    "operationKind": "query",
    "text": "query UserRepos_pinnedReposQuery(\n  $user: String!\n) {\n  repositoryOwner(login: $user) {\n    __typename\n    ... on ProfileOwner {\n      __isProfileOwner: __typename\n      pinnedItems(first: 6, types: [REPOSITORY]) {\n        nodes {\n          __typename\n          ... on Repository {\n            nameWithOwner\n            isPrivate\n            description\n            primaryLanguage {\n              name\n              color\n              id\n            }\n            stargazers {\n              totalCount\n            }\n            forkCount\n          }\n          ... on Node {\n            __isNode: __typename\n            id\n          }\n        }\n      }\n    }\n    id\n  }\n}\n"
  }
};
})();

(node as any).hash = "fa20133b463d3ca21426905883ce990a";

export default node;
