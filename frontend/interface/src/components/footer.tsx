import React from 'react';

const Footer: React.FC = () => {
  return (
    <footer className="text-center text-white text-sm py-8 bg-[#1D1D1D] border-t border-gray-700">
      © {new Date().getFullYear()} Under Control — All rights reserved.
    </footer>
  );
};

export default Footer;

