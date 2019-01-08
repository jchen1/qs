import React, { Component } from 'react';
import styled from 'styled-components';

import ApolloClient from 'apollo-boost';
import { ApolloProvider } from 'react-apollo';

import { violet, orange } from './constants';
import User from './User';

const client = new ApolloClient({
  uri: 'http://localhost:8080/graphql',
});

const StyledApp = styled.div`
  background-color: ${violet};
  color: ${orange};
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
