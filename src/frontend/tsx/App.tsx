import React, { Component } from 'react';
import logo from '../logo.svg';
import '../scss/App.css';

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
        user(id: "lol") {
          id
          email
        }
      }
    `}
  >
    {({ loading, error, data }) => {
      if (loading) return <p>Loading...</p>;
      if (error) return <p>Error :(</p>;

      return <div><p>{`${data.user.id}: ${data.user.email}`}</p></div>;
    }}
  </Query>
);

class App extends Component {
  render() {
    return (
      <ApolloProvider client={client}>
        <div className="App">
          <User />
        </div>
      </ApolloProvider>
    );
  }
}

export default App;
