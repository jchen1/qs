import React, { Component } from 'react';
import styled from 'styled-components';

import ApolloClient from 'apollo-boost';
import { ApolloProvider } from 'react-apollo';

import { violet, orange } from './constants';
import User from './User';
import Header from './Header';

const client = new ApolloClient({
  uri: 'http://localhost:8080/graphql',
});

const StyledApp = styled.div`
  background-color: ${violet};
  text-align: center;
  display: grid;
  min-height: 100vh;
  grid-template-rows: [header] 120px [content] auto;
`;

class App extends Component {
  render() {
    return (
      <ApolloProvider client={client}>
        <StyledApp>
          <Header />
          <User />
        </StyledApp>
      </ApolloProvider>
    );
  }
}

export default App;
