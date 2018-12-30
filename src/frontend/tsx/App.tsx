import React, { Component } from 'react';
import styled from 'styled-components';

import ApolloClient from "apollo-boost";
import { Query, ApolloProvider } from "react-apollo";
import gql from 'graphql-tag';

const client = new ApolloClient({
  uri: "http://localhost:8080/graphql"
});

const User = () => (
  <Query
    query={gql`
      {
        user {
          id
          email
          gSub
        }
      }
    `}
  >
    {({ loading, error, data }) => {
      if (loading) return <p>Loading...</p>;
      if (error) return <p>Error :(</p>;

      const { user } = data;
      if (user) return <div>
        <p>{`${data.user.id}: ${data.user.email}`}</p>
        <a href="oauth/fitbit/start">Fitbit</a>
      </div>;
      return <div><p>Anonymous - <a href="oauth/google/start">Login</a></p></div>
    }}
  </Query>
);

const StyledApp = styled.div`
  text-align: center;
`;

class App extends Component {
  render() {
    return (
      <ApolloProvider client={client}>
        <StyledApp>
          <User />
        </StyledApp>
      </ApolloProvider>
    );
  }
}

export default App;