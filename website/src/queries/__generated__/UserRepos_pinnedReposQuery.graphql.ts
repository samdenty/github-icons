/**
 * @generated SignedSource<<a19c42a8b3ce56cf69911624829351cf>>
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
        readonly nameWithOwner?: string;
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
  "kind": "InlineFragment",
  "selections": [
    {
      "alias": null,
      "args": null,
      "kind": "ScalarField",
      "name": "nameWithOwner",
      "storageKey": null
    }
  ],
  "type": "Repository",
  "abstractKey": null
},
v2 = {
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
                  (v1/*: any*/)
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
                  (v1/*: any*/),
                  {
                    "kind": "InlineFragment",
                    "selections": [
                      (v2/*: any*/)
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
          (v2/*: any*/)
        ],
        "storageKey": null
      }
    ]
  },
  "params": {
    "cacheID": "bd0515c2cd16de33188ad3b0d26f875c",
    "id": null,
    "metadata": {},
    "name": "UserRepos_pinnedReposQuery",
    "operationKind": "query",
    "text": "query UserRepos_pinnedReposQuery {\n  viewer {\n    pinnedItems(first: 6, types: REPOSITORY) {\n      nodes {\n        __typename\n        ... on Repository {\n          nameWithOwner\n        }\n        ... on Node {\n          __isNode: __typename\n          id\n        }\n      }\n    }\n    id\n  }\n}\n"
  }
};
})();

(node as any).hash = "4d8f09afed0494c7074b9ece96266865";

export default node;
