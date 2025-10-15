import './index.css';
import MarketCard from './components/MarketCard'
import MarketsPage from './components/Markets'
import Header from './components/Header'
import Footer from './components/footer'
import type {IndicatorKind, MarketInfo} from './types'
import viteLogo from '/vite.svg';


const handleTogglePause = (asset: string) => {
  console.log(`Toggled pause for ${asset}`);
};

const handleRemove = (asset: string) => {
  console.log(`Removed market ${asset}`);
};

const App: React.FC = () => (
    <div className= "bg-[#1D1D1D] h-full">
        <Header />
        <MarketsPage/>
        <Footer />
    </div>
);

export default App;


