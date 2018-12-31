const proxy = require('http-proxy-middleware');

function filter(pathname, req) {
  return pathname.match(/(graphi?ql)|(oauth\/?.*)|(logout)/);
}

module.exports = function(app) {
  app.use(proxy(filter, { target: 'http://localhost:8081/' }));
};