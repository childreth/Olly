// Car Inventory & Search Engine (CarGurus-style dataset)

export const VEHICLES = [
  {
    id: 'car-honda-accord-2022',
    vin: '1HGCV1F39NA018241',
    year: 2022,
    make: 'Honda',
    model: 'Accord',
    trim: 'Sport 1.5T',
    body_style: 'Sedan',
    price: 24900,
    market_average: 26800,
    deal_rating: 'Great Deal',
    deal_diff: -1900,
    mileage: 22400,
    exterior_color: 'Platinum White Pearl',
    interior_color: 'Black Cloth',
    transmission: 'CVT Automatic',
    drivetrain: 'FWD',
    fuel_type: 'Gasoline',
    mpg: '29 City / 35 Hwy',
    days_on_market: 18,
    history: {
      accidents: 0,
      owners: 1,
      clean_title: true,
      service_records: 6
    },
    features: [
      'Apple CarPlay / Android Auto',
      'Honda Sensing Suite (Adaptive Cruise, Lane Keeping)',
      '19-inch Alloy Wheels',
      'Dual-zone Climate Control',
      'Rearview Camera'
    ],
    dealer: {
      id: 'dealer-boston-honda',
      name: 'Metro Boston Honda',
      city: 'Cambridge',
      state: 'MA',
      zip: '02138',
      email: 'sales@metrobostonhonda-mock.com',
      phone: '(617) 555-0149',
      rating: 4.7,
      reviews_count: 428,
      typical_doc_fee: 495
    }
  },
  {
    id: 'car-toyota-rav4-2023',
    vin: '2T3P1RFV5PW109284',
    year: 2023,
    make: 'Toyota',
    model: 'RAV4',
    trim: 'XLE AWD',
    body_style: 'SUV',
    price: 28450,
    market_average: 29900,
    deal_rating: 'Good Deal',
    deal_diff: -1450,
    mileage: 18200,
    exterior_color: 'Magnetic Gray Metallic',
    interior_color: 'Ash SofTex',
    transmission: '8-Speed Automatic',
    drivetrain: 'AWD',
    fuel_type: 'Gasoline',
    mpg: '27 City / 33 Hwy',
    days_on_market: 32,
    history: {
      accidents: 0,
      owners: 1,
      clean_title: true,
      service_records: 4
    },
    features: [
      'Toyota Safety Sense 2.5',
      'Blind Spot Monitor with Rear Cross-Traffic Alert',
      'Power Moonroof',
      'Dual-Zone Automatic Climate',
      'All-Weather Floor Liners'
    ],
    dealer: {
      id: 'dealer-newengland-toyota',
      name: 'Woburn Premier Toyota',
      city: 'Woburn',
      state: 'MA',
      zip: '01801',
      email: 'internet-sales@woburntoyota-mock.com',
      phone: '(781) 555-0182',
      rating: 4.4,
      reviews_count: 512,
      typical_doc_fee: 599
    }
  },
  {
    id: 'car-tesla-model3-2022',
    vin: '5YJ3E1EB9NF293810',
    year: 2022,
    make: 'Tesla',
    model: 'Model 3',
    trim: 'Long Range AWD',
    body_style: 'Sedan',
    price: 26800,
    market_average: 28500,
    deal_rating: 'Great Deal',
    deal_diff: -1700,
    mileage: 31000,
    exterior_color: 'Deep Blue Metallic',
    interior_color: 'All Black Premium',
    transmission: '1-Speed Direct Drive',
    drivetrain: 'AWD',
    fuel_type: 'Electric',
    mpg: '358 mi EPA Range',
    days_on_market: 41,
    history: {
      accidents: 0,
      owners: 1,
      clean_title: true,
      battery_health: '94% State of Health'
    },
    features: [
      'Autopilot',
      'Dual Motor All-Wheel Drive',
      '15-inch Touchscreen Navigation',
      'Premium Audio (13 speakers)',
      'Heated Seats Front & Rear'
    ],
    dealer: {
      id: 'dealer-metro-ev',
      name: 'NorthEast EV Hub',
      city: 'Somerville',
      state: 'MA',
      zip: '02143',
      email: 'quotes@northeast-ev-mock.com',
      phone: '(617) 555-0199',
      rating: 4.8,
      reviews_count: 189,
      typical_doc_fee: 395
    }
  },
  {
    id: 'car-subaru-outback-2021',
    vin: '4S4BTANC7M3109482',
    year: 2021,
    make: 'Subaru',
    model: 'Outback',
    trim: 'Premium',
    body_style: 'Wagon',
    price: 21995,
    market_average: 22500,
    deal_rating: 'Fair Deal',
    deal_diff: -505,
    mileage: 44200,
    exterior_color: 'Autumn Green Metallic',
    interior_color: 'Warm Ivory Cloth',
    transmission: 'Lineartronic CVT',
    drivetrain: 'AWD',
    fuel_type: 'Gasoline',
    mpg: '26 City / 33 Hwy',
    days_on_market: 12,
    history: {
      accidents: 0,
      owners: 2,
      clean_title: true,
      service_records: 9
    },
    features: [
      'EyeSight Driver Assist',
      '11.6-inch Multimedia Display',
      'Starlink Safety & Security',
      'X-Mode with Hill Descent',
      'Roof Rails with Integrated Crossbars'
    ],
    dealer: {
      id: 'dealer-boston-subaru',
      name: 'Burlington Subaru Gallery',
      city: 'Burlington',
      state: 'MA',
      zip: '01803',
      email: 'leads@burlingtonsubaru-mock.com',
      phone: '(781) 555-0123',
      rating: 4.5,
      reviews_count: 367,
      typical_doc_fee: 495
    }
  },
  {
    id: 'car-ford-f150-2021',
    vin: '1FTFW1ED6MFC91823',
    year: 2021,
    make: 'Ford',
    model: 'F-150',
    trim: 'Lariat SuperCrew 4x4',
    body_style: 'Pickup Truck',
    price: 37900,
    market_average: 41200,
    deal_rating: 'Great Deal',
    deal_diff: -3300,
    mileage: 39500,
    exterior_color: 'Agate Black',
    interior_color: 'Black Leather',
    transmission: '10-Speed Automatic',
    drivetrain: '4WD',
    fuel_type: '3.5L EcoBoost V6 Gas',
    mpg: '18 City / 23 Hwy',
    days_on_market: 54,
    history: {
      accidents: 0,
      owners: 1,
      clean_title: true,
      service_records: 8
    },
    features: [
      'SYNC 4 with 12-inch Touchscreen',
      'B&O Sound System',
      'Ford Co-Pilot360 2.0',
      'Heated and Ventilated Front Seats',
      'Class IV Trailer Hitch'
    ],
    dealer: {
      id: 'dealer-middlesex-ford',
      name: 'Middlesex Ford Commercial & Truck',
      city: 'Lowell',
      state: 'MA',
      zip: '01852',
      email: 'sales@middlesexford-mock.com',
      phone: '(978) 555-0164',
      rating: 4.2,
      reviews_count: 245,
      typical_doc_fee: 649
    }
  },
  {
    id: 'car-mazda-cx5-2022',
    vin: 'JM3KFBDM8N0589123',
    year: 2022,
    make: 'Mazda',
    model: 'CX-5',
    trim: '2.5 S Preferred AWD',
    body_style: 'SUV',
    price: 23500,
    market_average: 24400,
    deal_rating: 'Good Deal',
    deal_diff: -900,
    mileage: 26100,
    exterior_color: 'Soul Red Crystal Metallic',
    interior_color: 'Black Leather',
    transmission: '6-Speed Automatic',
    drivetrain: 'AWD',
    fuel_type: 'Gasoline',
    mpg: '24 City / 30 Hwy',
    days_on_market: 23,
    history: {
      accidents: 0,
      owners: 1,
      clean_title: true,
      service_records: 5
    },
    features: [
      'Power Moonroof',
      'Bose 10-Speaker Audio',
      'Dual-zone Climate Control',
      'i-Activsense Safety Suite',
      'Power Liftgate'
    ],
    dealer: {
      id: 'dealer-boston-honda', // same group
      name: 'Metro Boston Auto Group',
      city: 'Cambridge',
      state: 'MA',
      zip: '02138',
      email: 'sales@metrobostonhonda-mock.com',
      phone: '(617) 555-0149',
      rating: 4.7,
      reviews_count: 428,
      typical_doc_fee: 495
    }
  }
];

export function searchCars(filters = {}) {
  let results = [...VEHICLES];

  if (filters.make) {
    const q = filters.make.toLowerCase().trim();
    results = results.filter(c => c.make.toLowerCase().includes(q));
  }

  if (filters.model) {
    const q = filters.model.toLowerCase().trim();
    results = results.filter(c => c.model.toLowerCase().includes(q));
  }

  if (filters.min_price !== undefined && filters.min_price !== null) {
    results = results.filter(c => c.price >= Number(filters.min_price));
  }

  if (filters.max_price !== undefined && filters.max_price !== null) {
    results = results.filter(c => c.price <= Number(filters.max_price));
  }

  if (filters.max_mileage !== undefined && filters.max_mileage !== null) {
    results = results.filter(c => c.mileage <= Number(filters.max_mileage));
  }

  if (filters.min_year !== undefined && filters.min_year !== null) {
    results = results.filter(c => c.year >= Number(filters.min_year));
  }

  if (filters.deal_rating) {
    const q = filters.deal_rating.toLowerCase().trim();
    results = results.filter(c => c.deal_rating.toLowerCase().includes(q));
  }

  if (filters.body_style) {
    const q = filters.body_style.toLowerCase().trim();
    results = results.filter(c => c.body_style.toLowerCase().includes(q));
  }

  // Sort by deal quality or price if specified
  if (filters.sort === 'price_asc') {
    results.sort((a, b) => a.price - b.price);
  } else if (filters.sort === 'price_desc') {
    results.sort((a, b) => b.price - a.price);
  } else if (filters.sort === 'mileage_asc') {
    results.sort((a, b) => a.mileage - b.mileage);
  } else {
    // Default: best deal first
    results.sort((a, b) => a.deal_diff - b.deal_diff);
  }

  return results;
}

export function getCarById(id) {
  return VEHICLES.find(c => c.id === id || c.vin === id) || null;
}
